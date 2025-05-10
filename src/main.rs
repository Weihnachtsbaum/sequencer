#![feature(register_tool)]
#![register_tool(bevy)]
#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use std::{
    fmt::{self, Display},
    mem,
};

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::{PrimaryWindow, WindowResized},
    winit::WinitSettings,
};

const CELL_WIDTH: f32 = 50.0;
const CELL_HEIGHT: f32 = 25.0;
const LINE_WIDTH: f32 = 1.0;
const LINE_LEN: f32 = 10000.0;
const NUM_SEMIS: u8 = 120;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, (on_window_resize, scroll))
        .run()
}

struct Semi(u8);

impl Display for Semi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letters = match self.0 % 12 {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => unreachable!(),
        };
        let octave = self.0 / 12;
        write!(f, "{letters}{octave}")
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(
            window.size().x / 2.0,
            NUM_SEMIS as f32 * CELL_HEIGHT / 2.0,
            0.0,
        ),
    ));

    let mesh = Mesh2d(meshes.add(Rectangle::default()));
    let material = MeshMaterial2d(materials.add(Color::BLACK));
    let x_cells = (window.size().x / CELL_WIDTH) as i32;

    for x in 1..x_cells + 1 {
        commands.spawn((
            mesh.clone(),
            material.clone(),
            Transform {
                translation: Vec3::new(x as f32 * CELL_WIDTH, 0.0, 0.0),
                scale: Vec3::new(LINE_WIDTH, LINE_LEN, 1.0),
                ..default()
            },
        ));
    }
    for semi in 0..NUM_SEMIS {
        let line_y = semi as f32 * CELL_HEIGHT;
        commands.spawn((
            mesh.clone(),
            material.clone(),
            Transform {
                translation: Vec3::new(0.0, line_y, 0.0),
                scale: Vec3::new(LINE_LEN, LINE_WIDTH, 1.0),
                ..default()
            },
        ));
        commands.spawn((
            Text2d(Semi(semi).to_string()),
            Transform::from_xyz(20.0, line_y + 0.5 * CELL_HEIGHT, 1.0),
        ));
    }
}

fn on_window_resize(
    mut evr: EventReader<WindowResized>,
    mut cam: Single<&mut Transform, With<Camera>>,
) {
    for ev in evr.read() {
        clamp_cam_pos(&mut cam.translation, Vec2::new(ev.width, ev.height));
    }
}

fn scroll(
    mut mouse_wheel_evr: EventReader<MouseWheel>,
    kb: Res<ButtonInput<KeyCode>>,
    mut cam: Single<&mut Transform, With<Camera>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for ev in mouse_wheel_evr.read() {
        let (mut dx, mut dy) = match ev.unit {
            MouseScrollUnit::Line => (ev.x * CELL_HEIGHT, ev.y * CELL_HEIGHT),
            MouseScrollUnit::Pixel => (ev.x, ev.y),
        };
        if kb.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            mem::swap(&mut dx, &mut dy);
        }
        cam.translation.x -= dx;
        cam.translation.y += dy;
        clamp_cam_pos(&mut cam.translation, window.size());
    }
}

fn clamp_cam_pos(pos: &mut Vec3, window_size: Vec2) {
    pos.x = pos.x.max(window_size.x / 2.0);
    pos.y = pos.y.clamp(
        window_size.y / 2.0,
        NUM_SEMIS as f32 * CELL_HEIGHT - window_size.y / 2.0,
    );
}
