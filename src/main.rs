#![feature(register_tool)]
#![register_tool(bevy)]
#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use std::{
    f32::consts::TAU,
    fmt::{self, Display},
    mem,
    time::Duration,
};

use bevy::{
    audio::{AddAudioSource, Source},
    input::{
        common_conditions::input_just_pressed,
        mouse::{MouseScrollUnit, MouseWheel},
    },
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
        .add_audio_source::<Track>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_window_resize,
                scroll,
                place_note.run_if(input_just_pressed(MouseButton::Left)),
                play.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .run()
}

#[derive(Resource)]
struct ColumnCount(u32);

#[derive(Resource)]
struct RectMesh(Handle<Mesh>);

#[derive(Resource)]
struct LineMaterial(Handle<ColorMaterial>);

#[derive(Resource)]
struct NoteMaterial(Handle<ColorMaterial>);

#[derive(Clone)]
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

impl Semi {
    const A4: Self = Self(57);
    const A4_HZ: f32 = 440.0;

    /// Returns the frequency of the semitone in
    /// [12-tone equal temperament](https://en.wikipedia.org/wiki/12_equal_temperament).
    fn hz(&self) -> f32 {
        let diff = self.0 as i32 - Self::A4.0 as i32;
        Self::A4_HZ * 2.0f32.powf(1.0 / 12.0).powi(diff)
    }
}

#[derive(Resource)]
struct TrackHandle(Handle<Track>);

#[derive(Asset, TypePath, Default)]
struct Track(Vec<Vec<Semi>>);

impl Decodable for Track {
    type Decoder = TrackDecoder;
    type DecoderItem = <Self::Decoder as Iterator>::Item;

    fn decoder(&self) -> Self::Decoder {
        TrackDecoder {
            notes: self.0.clone(),
            bpm: 120.0,
            sample: 0,
        }
    }
}

struct TrackDecoder {
    notes: Vec<Vec<Semi>>,
    bpm: f32,
    sample: u32,
}

impl Iterator for TrackDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let bps = self.bpm / 60.0;
        let sec = self.sample as f32 / self.sample_rate() as f32;
        let x = (bps * sec) as usize;
        let v = self
            .notes
            .get(x)?
            .iter()
            .map(|semi| (semi.hz() * TAU * sec).sin())
            .sum();
        self.sample += 1;
        Some(v)
    }
}

impl Source for TrackDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        41100
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.bpm / 60.0 * self.notes.len() as f32,
        ))
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tracks: ResMut<Assets<Track>>,
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
    commands.insert_resource(RectMesh(mesh.0.clone()));
    let material = MeshMaterial2d(materials.add(Color::BLACK));
    commands.insert_resource(LineMaterial(material.0.clone()));
    let x_cells = (window.size().x / CELL_WIDTH) as u32;
    commands.insert_resource(ColumnCount(x_cells));

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

    commands.insert_resource(NoteMaterial(materials.add(Color::srgb(0.7, 0.2, 0.4))));

    commands.insert_resource(TrackHandle(tracks.add(Track::default())));
}

fn on_window_resize(
    mut evr: EventReader<WindowResized>,
    mut cam: Single<&mut Transform, With<Camera>>,
    mut commands: Commands,
) {
    for ev in evr.read() {
        clamp_cam_pos(&mut cam.translation, Vec2::new(ev.width, ev.height));
        commands.run_system_cached(update_column_count);
    }
}

fn scroll(
    mut mouse_wheel_evr: EventReader<MouseWheel>,
    kb: Res<ButtonInput<KeyCode>>,
    mut cam: Single<&mut Transform, With<Camera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
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
        commands.run_system_cached(update_column_count);
    }
}

fn clamp_cam_pos(pos: &mut Vec3, window_size: Vec2) {
    pos.x = pos.x.max(window_size.x / 2.0);
    pos.y = pos.y.clamp(
        window_size.y / 2.0,
        NUM_SEMIS as f32 * CELL_HEIGHT - window_size.y / 2.0,
    );
}

fn update_column_count(
    mut column_count: ResMut<ColumnCount>,
    cam: Single<&Transform, With<Camera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    line_mesh: Res<RectMesh>,
    line_material: Res<LineMaterial>,
) {
    let new_count = ((cam.translation.x + window.size().x / 2.0) / CELL_WIDTH) as u32;
    for x in column_count.0 + 1..new_count + 1 {
        commands.spawn((
            Mesh2d(line_mesh.0.clone()),
            MeshMaterial2d(line_material.0.clone()),
            Transform {
                translation: Vec3::new(x as f32 * CELL_WIDTH, 0.0, 0.0),
                scale: Vec3::new(LINE_WIDTH, LINE_LEN, 1.0),
                ..default()
            },
        ));
    }
    if new_count > column_count.0 {
        column_count.0 = new_count;
    }
}

fn place_note(
    window: Single<&Window, With<PrimaryWindow>>,
    cam: Single<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mesh: Res<RectMesh>,
    material: Res<NoteMaterial>,
    track: Res<TrackHandle>,
    mut tracks: ResMut<Assets<Track>>,
) -> Result {
    let pos = window.cursor_position().ok_or("No cursor pos")?;
    let (cam, cam_transform) = *cam;
    let pos = cam.viewport_to_world_2d(cam_transform, pos)?;
    let scale = Vec2::new(CELL_WIDTH, CELL_HEIGHT);
    let grid_pos = (pos / scale).floor();
    if grid_pos.x < 1.0 {
        return Ok(());
    }
    commands.spawn((
        Mesh2d(mesh.0.clone()),
        MeshMaterial2d(material.0.clone()),
        Transform {
            translation: ((grid_pos + Vec2::splat(0.5)) * scale).extend(-1.0),
            scale: scale.extend(1.0),
            ..default()
        },
    ));
    let x = grid_pos.x as usize - 1;
    let semi = Semi(grid_pos.y as u8);
    let track = tracks.get_mut(track.0.id()).ok_or("No track")?;
    if track.0.len() <= x {
        track.0.resize_with(x + 1, Default::default);
    }
    track.0[x].push(semi);
    Ok(())
}

fn play(mut commands: Commands, track: Res<TrackHandle>) {
    commands.spawn((AudioPlayer(track.0.clone()), PlaybackSettings::DESPAWN));
}
