use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::{MouseWheel, MouseMotion};
use bevy::prelude::*;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,

    /// How many times faster to move with shift held down?
    pub boost: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
            boost: 4.,
        }
    }
}

/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
#[cfg(not(target_family="wasm"))]
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts (only works for non-wasm)
#[cfg(not(target_family="wasm"))]
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        toggle_grab_cursor(window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);
}

/// Returns the amount to boost or slow down by. (shift = run)
fn get_boost(keys: &Input<KeyCode>, settings: &MovementSettings) -> f32 {
    let mut boost = 1.;
    for key in keys.get_pressed() {
        match key {
            KeyCode::LShift => boost = settings.boost,
            KeyCode::O => boost = 1. / settings.boost, // slow motion mode
            _ => (),
        }
    }
    boost
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    settings: Res<MovementSettings>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    for mut transform in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);
        let boost = get_boost(&keys, &settings);
        let mut rx = 0.;
        let mut ry = 0.;
        let mut rz = 0.;

        for key in keys.get_pressed() {
            match key {
                KeyCode::W | KeyCode::Up => velocity += forward,
                KeyCode::S | KeyCode::Down => velocity -= forward,
                KeyCode::A | KeyCode::Left => velocity -= right,
                KeyCode::D | KeyCode::Right => velocity += right,
                KeyCode::Space | KeyCode::Period => velocity += Vec3::Y,
                KeyCode::RShift | KeyCode::Comma => velocity -= Vec3::Y,
                KeyCode::LBracket => {
                    ry -= time.delta_seconds();
                } // yaw, pitch, roll.
                KeyCode::RBracket => {
                    ry += time.delta_seconds();
                }
                KeyCode::Q => {
                    rx -= time.delta_seconds();
                } // yaw, pitch, roll.
                KeyCode::E => {
                    rx += time.delta_seconds();
                }
                KeyCode::Z => {
                    rz -= time.delta_seconds();
                } // yaw, pitch, roll.
                KeyCode::X => {
                    rz += time.delta_seconds();
                }
                // Note: bevy 0.7 bug: if you press LShift and then Comma no additional key seems to be pressed
                _ => (),
            }
        }

        velocity = velocity.normalize_or_zero();

        transform.translation += velocity * time.delta_seconds() * settings.speed * boost;

        let delta_x = settings.speed * boost * rx / 100. * std::f32::consts::PI * 2.0;
        let delta_y = settings.speed * boost * ry / 100. * std::f32::consts::PI;
        let delta_z = settings.speed * boost * rz / 100. * std::f32::consts::PI;
        let yaw = Quat::from_rotation_y(-delta_x);
        let pitch = Quat::from_rotation_x(-delta_y);
        let roll = Quat::from_rotation_z(-delta_z);
        transform.rotation = yaw * transform.rotation; // rotate around global y axis
        transform.rotation = transform.rotation * pitch * roll; // rotate around local x axis
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
    buttons: Res<Input<MouseButton>>,
) {
    if let Some(window) = windows.get_primary() {
        let please_move = buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Right);

        #[cfg(target_arch = "wasm32")]
        {
            let browser_window = web_sys::window().expect("no global `window` exists");
            let document = browser_window
                .document()
                .expect("should have a document on window");
            let locked = document.pointer_lock_element().is_some();
            if !locked && !please_move {
                return;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if !window.cursor_locked() && !please_move {
            return;
        }

        for mut transform in query.iter_mut() {
            for ev in state.reader_motion.iter(&motion) {
                let window_scale = window.height().min(window.width());

                // Order is important to prevent unintended roll
                let yaw = Quat::from_rotation_y(
                    -(settings.sensitivity * ev.delta.x * window_scale).to_radians(),
                );
                let pitch = Quat::from_rotation_x(
                    -(settings.sensitivity * ev.delta.y * window_scale).to_radians(),
                );
                transform.rotation = yaw * transform.rotation; // rotate around global y axis
                transform.rotation *= pitch; // rotate around local x axis
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

/// Long running processes are not allowed to grab the cursor in wasm - this must be done by
/// some user activated short lived action. (see index.html)
#[cfg(not(target_family="wasm"))]
fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// the mouse-scroll does not change the field-of-view of the camera
/// because if you change that too far the world goes inside out.
/// Instead scroll moves forwards or backwards.
pub fn scroll(
	settings: Res<MovementSettings>,
    keys: Res<Input<KeyCode>>,
	mut mouse_wheel_events: EventReader<MouseWheel>,
	mut query: Query<&mut Transform, With<FlyCam>>,
) {
	for event in mouse_wheel_events.iter() {
		for mut viewport in query.iter_mut() {
            // In browser this seems a lot more sensitive!
			#[cfg(target_arch = "wasm32")]
			let sensitivity: f32 = settings.sensitivity * 10.0;
			#[cfg(not(target_arch = "wasm32"))]
			let sensitivity: f32 = settings.sensitivity * 1024.0;
            let forward = viewport.forward();
			viewport.translation += forward * event.y * sensitivity * get_boost(&keys, &settings);
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
            .add_system(player_move)
            .add_system(player_look)
            .add_system(scroll);

        #[cfg(not(target_family="wasm"))]
        app.add_startup_system(initial_grab_cursor)
            .add_system(cursor_grab);
    }
}

/// Same as [`PlayerPlugin`] but does not spawn a camera
pub struct NoCameraPlayerPlugin;
impl Plugin for NoCameraPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_system(player_move)
            .add_system(player_look)
            .add_system(scroll);

        #[cfg(not(target_family="wasm"))]
        app.add_startup_system(initial_grab_cursor)
            .add_system(cursor_grab);
    }
}
