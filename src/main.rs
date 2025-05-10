#![feature(register_tool)]
#![register_tool(bevy)]
#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use std::fmt::{self, Display};

use bevy::{prelude::*, window::PrimaryWindow};

const CELL_WIDTH: f32 = 50.0;
const CELL_HEIGHT: f32 = 25.0;
const LINE_WIDTH: f32 = 1.0;
const LINE_LEN: f32 = 10000.0;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
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
    commands.spawn(Camera2d);

    let mesh = Mesh2d(meshes.add(Rectangle::default()));
    let material = MeshMaterial2d(materials.add(Color::BLACK));
    let x_cells = (window.size().x / CELL_WIDTH) as i32;
    let y_cells = (window.size().y / CELL_HEIGHT) as i32;

    for x in -x_cells / 2 - 1..x_cells / 2 + 1 {
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
    for y in -y_cells / 2 - 1..y_cells / 2 + 1 {
        commands.spawn((
            mesh.clone(),
            material.clone(),
            Transform {
                translation: Vec3::new(0.0, y as f32 * CELL_HEIGHT, 0.0),
                scale: Vec3::new(LINE_LEN, LINE_WIDTH, 1.0),
                ..default()
            },
        ));
        let semi = y + y_cells / 2 + 10;
        commands.spawn((
            Text2d(Semi(semi as _).to_string()),
            Transform::from_xyz(
                window.size().x / -2.0 + 20.0,
                (y as f32 + 0.5) * CELL_HEIGHT,
                1.0,
            ),
        ));
    }
}
