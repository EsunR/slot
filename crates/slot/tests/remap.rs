mod common;

use slot::app::{App, GameMenu, Phase};
use slot::input::{Pad, Remap};
use slot::root;
use slot::session::Session;
use slot_input::{Action, Btn, RawEvent};
use slot_retro::ButtonMask;
use slot_store::{write_slot_state, Core, SlotState, System};
use slot_ui::{lang, menu_face, Draw, TexId, OUT_H, OUT_W};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn playing() -> (App, TempDir) {
    let d = common::tmp_root_with_carts(&["Emerald", "Zzz"]);
    let mut app = common::boot(d.path());
    app.apply(Action::Insert);
    app.set_core(Core::Mgba);
    app.on_core_ready();
    for _ in 0..120 {
        app.update(1.0 / 60.0);
    }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    (app, d)
}

fn press(app: &mut App, b: Btn, times: usize) {
    for _ in 0..times {
        app.apply(Action::GbaDown(b));
    }
}

fn edit(app: &mut App) {
    app.apply(Action::GameMenu);
    press(app, Btn::A, 1);
    assert_eq!(app.game_menu(), Some(GameMenu::Remap));
}

#[test]
fn per_game_files_inherit_defaults_and_are_isolated_by_game_and_machine() {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    std::fs::write(
        d.path().join("System/remap.txt"),
        "invalid line\nX = A\nB = A\n",
    )
    .unwrap();
    assert_eq!(
        root::game_remap(d.path(), System::Gba, "Emerald").target(Btn::X),
        Btn::A
    );
    let dir = d.path().join("System/Remaps/GBA");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Emerald.txt"), "B=B\nbad\nY=A\n").unwrap();
    let gba = root::game_remap(d.path(), System::Gba, "Emerald");
    assert_eq!(gba.target(Btn::X), Btn::A);
    assert_eq!(gba.target(Btn::B), Btn::B);
    assert_eq!(gba.target(Btn::Y), Btn::A);
    for (system, stem) in [
        (System::Gb, "Emerald"),
        (System::Gbc, "Emerald"),
        (System::Gba, "Zzz"),
    ] {
        assert_eq!(
            root::game_remap(d.path(), system, stem).target(Btn::B),
            Btn::A
        );
    }
    root::write_game_remap(d.path(), System::Gba, "Emerald", gba).unwrap();
    // Atomic replacement must work when the file already exists.
    root::write_game_remap(d.path(), System::Gba, "Emerald", gba).unwrap();
    assert_eq!(root::game_remap(d.path(), System::Gba, "Emerald"), gba);
}

#[test]
fn saving_a_draft_persists_it_and_emits_the_live_layout() {
    let (mut app, d) = playing();
    edit(&mut app);
    press(&mut app, Btn::Down, 6); // X
    press(&mut app, Btn::Right, 5); // disabled -> UP -> DOWN -> LEFT -> RIGHT -> A
    assert!(app.remap_menu_lines().iter().any(|l| l == "X  >  A"));
    assert!(app.take_remap_changed().is_none());
    press(&mut app, Btn::Down, 6); // Save
    press(&mut app, Btn::A, 1);
    assert!(!app.game_menu_open());
    let live = app.take_remap_changed().expect("save must reach Session");
    assert_eq!(live.target(Btn::X), Btn::A);
    assert_eq!(root::game_remap(d.path(), System::Gba, "Emerald"), live);
    edit(&mut app);
    press(&mut app, Btn::Down, 6);
    assert!(app.remap_menu_lines().iter().any(|l| l == "X  >  A"));
}

#[test]
fn cancelling_discards_changes_and_restoring_defaults_removes_the_override() {
    let (mut app, d) = playing();
    std::fs::write(d.path().join("System/remap.txt"), "X=A\n").unwrap();
    edit(&mut app);
    press(&mut app, Btn::Right, 1);
    press(&mut app, Btn::B, 1);
    assert_eq!(app.game_menu(), Some(GameMenu::Rows(0)));
    assert!(app.take_remap_changed().is_none());
    assert!(!d.path().join("System/Remaps/GBA/Emerald.txt").exists());
    press(&mut app, Btn::B, 1);
    let mut custom = Remap::default();
    custom.set(Btn::X, Btn::B);
    root::write_game_remap(d.path(), System::Gba, "Emerald", custom).unwrap();
    edit(&mut app);
    press(&mut app, Btn::Up, 1); // wrap to Restore defaults
    press(&mut app, Btn::A, 1);
    assert!(!app.game_menu_open());
    assert_eq!(app.take_remap_changed().unwrap().target(Btn::X), Btn::A);
    assert!(!d.path().join("System/Remaps/GBA/Emerald.txt").exists());
}

#[test]
fn a_write_failure_keeps_the_draft_for_retry() {
    let (mut app, d) = playing();
    std::fs::write(d.path().join("System/Remaps"), "blocks directory creation").unwrap();
    edit(&mut app);
    press(&mut app, Btn::Right, 1); // UP -> DOWN
    press(&mut app, Btn::Up, 2); // Save
    press(&mut app, Btn::A, 1);
    assert_eq!(app.game_menu(), Some(GameMenu::Remap));
    assert!(app
        .remap_menu_lines()
        .iter()
        .any(|l| l == lang::REMAP_SAVE_FAILED));
    assert!(app.take_remap_changed().is_none());
    std::fs::remove_file(d.path().join("System/Remaps")).unwrap();
    press(&mut app, Btn::A, 1);
    assert_eq!(app.take_remap_changed().unwrap().target(Btn::Up), Btn::Down);
}

#[test]
fn replacing_a_layout_clears_held_keys_and_shared_targets_release_correctly() {
    let mut pad = Pad::default();
    pad.apply(Action::GbaDown(Btn::B));
    let mut layout = Remap::default();
    layout.set(Btn::X, Btn::A);
    pad.set_remap(layout);
    assert_eq!(pad.mask(), ButtonMask(0));
    pad.apply(Action::GbaDown(Btn::A));
    pad.apply(Action::GbaDown(Btn::X));
    pad.apply(Action::GbaUp(Btn::A));
    assert_eq!(pad.mask(), ButtonMask(ButtonMask::A));
    pad.apply(Action::GbaUp(Btn::X));
    assert_eq!(pad.mask(), ButtonMask(0));
}

#[test]
fn every_editor_page_draws_its_selection_within_the_screen() {
    // Chinese builds use the card's font. Validate that font too when a local
    // device package is available; the embedded Open Sans only covers Latin.
    let card_font = root::font_file(&common::repo_root().join("target-rg35xxsp-sdcard"));
    if let Some(path) = &card_font {
        slot_ui::text::set_font(std::fs::read(path).unwrap());
    }
    let (mut app, _d) = playing();
    edit(&mut app);
    for _ in 0..14 {
        let lines = app.remap_menu_lines();
        let faces: Vec<_> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if card_font.is_some() || cfg!(feature = "lang-en") {
                    assert!(
                        slot_ui::text::missing_in(line).is_empty(),
                        "missing glyphs in {line}"
                    );
                }
                let face = menu_face(line);
                assert!(face.w <= OUT_W - 32, "line too wide: {line} ({})", face.w);
                (TexId::from_raw(900 + i), face.w, face.h)
            })
            .collect();
        app.set_remap_faces(lines, faces.clone());
        let mut draws = Vec::new();
        app.draw(&mut draws);
        let selected = draws.iter().filter(|d| matches!(d, Draw::Rect { colour, w, .. } if *colour == slot_ui::edge() && *w < OUT_W as f32)).count();
        assert!(selected >= 1);
        for d in draws {
            if let Draw::Tex {
                tex, x, y, w, h, ..
            } = d
            {
                if faces.iter().any(|f| f.0 == tex) {
                    assert!(x >= 0.0 && y >= 0.0 && x + w <= OUT_W as f32 && y + h <= OUT_H as f32);
                }
            }
        }
        press(&mut app, Btn::Down, 1);
    }
}

#[test]
fn saved_mapping_loads_at_boot_and_menu_save_updates_the_running_core() {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let mut layout = Remap::default();
    layout.set(Btn::X, Btn::B);
    root::write_game_remap(d.path(), System::Gba, "Emerald", layout).unwrap();
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Emerald".into()),
            clock_set: true,
            utc_offset_min: 0,
            ..Default::default()
        },
    )
    .unwrap();
    let mut session = Session::boot(d.path().to_path_buf());
    let mut now = 0;
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(session.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline);
        now += 16;
        session.feed([], now);
        session.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut send = |s: &mut Session, events: &[RawEvent]| {
        now += 16;
        s.feed(events.iter().copied(), now);
        s.update(1.0 / 60.0);
    };
    send(&mut session, &[RawEvent::Down(Btn::X)]);
    assert_eq!(session.emu().unwrap().input(), ButtonMask(ButtonMask::B));
    send(
        &mut session,
        &[
            RawEvent::Up(Btn::X),
            RawEvent::Down(Btn::Select),
            RawEvent::Down(Btn::Menu),
        ],
    );
    send(
        &mut session,
        &[RawEvent::Up(Btn::Menu), RawEvent::Up(Btn::Select)],
    );
    send(
        &mut session,
        &[RawEvent::Down(Btn::A), RawEvent::Up(Btn::A)],
    );
    assert_eq!(session.app().game_menu(), Some(GameMenu::Remap));
    for _ in 0..6 {
        send(
            &mut session,
            &[RawEvent::Down(Btn::Down), RawEvent::Up(Btn::Down)],
        );
    }
    send(
        &mut session,
        &[RawEvent::Down(Btn::Left), RawEvent::Up(Btn::Left)],
    ); // X: B -> A
    for _ in 0..6 {
        send(
            &mut session,
            &[RawEvent::Down(Btn::Down), RawEvent::Up(Btn::Down)],
        );
    }
    send(
        &mut session,
        &[RawEvent::Down(Btn::A), RawEvent::Up(Btn::A)],
    );
    assert!(!session.app().game_menu_open());
    assert_eq!(session.emu().unwrap().input(), ButtonMask(0));
    send(&mut session, &[RawEvent::Down(Btn::X)]);
    assert_eq!(session.emu().unwrap().input(), ButtonMask(ButtonMask::A));
    assert_eq!(
        root::game_remap(d.path(), System::Gba, "Emerald").target(Btn::X),
        Btn::A
    );
}
