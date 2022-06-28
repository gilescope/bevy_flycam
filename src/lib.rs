use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,

    /// How many times faster to move with shift held down?
    pub boost: f32
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
            boost: 4.
        }
    }
}

/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        toggle_grab_cursor(window);
    }
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Some(window) = windows.get_primary() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);
            let mut boost = 1.;
            let mut rx = 0.;
            let mut ry = 0.;
            let mut rz = 0.;

            for key in keys.get_pressed() {
                if window.cursor_locked() {
                    match key {
                        KeyCode::W | KeyCode::Up => velocity += forward,
                        KeyCode::S | KeyCode::Down => velocity -= forward,
                        KeyCode::A | KeyCode::Left => velocity -= right,
                        KeyCode::D | KeyCode::Right => velocity += right,
                        KeyCode::Space | KeyCode::Period => velocity += Vec3::Y,
                        KeyCode::RShift | KeyCode::Comma => velocity -= Vec3::Y,
                        KeyCode::LShift => boost = settings.boost,
                        KeyCode::O => boost = 1. / settings.boost, // slow motion mode
                        KeyCode::LBracket => { ry -= time.delta_seconds(); }, // yaw, pitch, roll.
                        KeyCode::RBracket => { ry += time.delta_seconds(); },
                        KeyCode::Q => { rx -= time.delta_seconds(); }, // yaw, pitch, roll.
                        KeyCode::E => { rx += time.delta_seconds(); },
                        KeyCode::Z => { rz -= time.delta_seconds(); }, // yaw, pitch, roll.
                        KeyCode::X => { rz += time.delta_seconds(); },
                        // Note: bevy 0.7 bug: if you press LShift and then Comma no additional key seems to be pressed
                        _ => (),
                    }
                }
            }
            
            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_seconds() * settings.speed * boost;

            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta =  settings.speed * boost * rx / 10. * std::f32::consts::PI * 2.0;
                // if pan_orbit.upside_down { -delta } else { delta }
                delta
            };
            let delta_y = settings.speed * boost * ry / 100. * std::f32::consts::PI;
            let delta_z = settings.speed * boost * rz / 100. * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            let roll = Quat::from_rotation_z(-delta_z);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch * roll; // rotate around local x axis
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}


/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Some(window) = windows.get_primary() {
        let mut delta_state = state.as_mut();
        for mut transform in query.iter_mut() {
            for ev in delta_state.reader_motion.iter(&motion) {
                if window.cursor_locked() {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.height().min(window.width());
                    delta_state.pitch -=
                        (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    delta_state.yaw -=
                        (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                }

                delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw)
                    * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
            }
        }
    }
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}

/// Same as [`PlayerPlugin`] but does not spawn a camera
pub struct NoCameraPlayerPlugin;
impl Plugin for NoCameraPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}
