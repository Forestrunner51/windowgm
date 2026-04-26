use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_tnua::builtins::{
    TnuaBuiltinCrouch, TnuaBuiltinCrouchConfig, TnuaBuiltinWalk, TnuaBuiltinWalkConfig,
};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // We need both Tnua's main controller plugin, and the plugin to connect to the physics
            // backend (in this case Avian 3D)
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_systems(Startup, (setup, setup_player))
        .add_systems(Update, apply_controls.in_set(TnuaUserControlsSystems))
        .run();
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum ControlScheme {
    Crouch(TnuaBuiltinCrouch),
}
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut control_scheme_configs: ResMut<Assets<ControlSchemeConfig>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d {
            radius: 0.5,
            half_length: 0.5,
        })),
        MeshMaterial3d(materials.add(Color::from(css::DARK_CYAN))),
        Transform::from_xyz(0.0, 2.0, 0.0),
        // The player character needs to be configured as a dynamic rigid body of the physics
        // engine.
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        // This is Tnua's interface component.
        TnuaController::<ControlScheme>::default(),
        // The configuration asset can be loaded from a file, but here we are creating it by code
        // and injecting it to the assets resource.
        TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                // The `float_height` must be greater (even if by little) from the distance between
                // the character's center and the lowest point of its collider.
                float_height: 1.5,
                // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they
                // have sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn
                // what they do.
                ..Default::default()
            },
            crouch: TnuaBuiltinCrouchConfig {
                float_offset: -0.4,
                ..Default::default()
            },
        })),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED,
    ));
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController<ControlScheme>>,
) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };
    // Note that we are not calling initiate_action_feeding

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X;
    }

    // Set the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO` to reset the previous frame's input.
    controller.basis = TnuaBuiltinWalk {
        // The `desired_motion` determines how the character will move.
        desired_motion: direction.normalize_or_zero(),
        // The other field is `desired_forward` - but since the character model is a capsule we
        // don't care the direction its "forward" is pointing.
        ..Default::default()
    };

    // Only need to feed the action while the button is pressed
    if keyboard.just_pressed(KeyCode::Space) {
        controller.action_start(ControlScheme::Crouch(Default::default()));
    }
    if keyboard.just_released(KeyCode::Space) {
        controller.action_end(ControlSchemeActionDiscriminant::Crouch);
    }
}
