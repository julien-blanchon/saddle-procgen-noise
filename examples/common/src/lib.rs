use bevy::{app::AppExit, prelude::*};

#[derive(Resource)]
struct ExampleAutoExit(Timer);

pub fn apply_window_defaults(
    app: &mut App,
    title: &str,
    resolution: (u32, u32),
    clear_color: Color,
) {
    app.insert_resource(ClearColor(clear_color))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.into(),
                resolution: resolution.into(),
                ..default()
            }),
            ..default()
        }));

    if let Some(timer) = auto_exit_from_env() {
        app.insert_resource(timer)
            .add_systems(Update, auto_exit_after_delay);
    }
}

fn auto_exit_from_env() -> Option<ExampleAutoExit> {
    let seconds = std::env::var("NOISE_EXAMPLE_EXIT_AFTER_SECONDS")
        .ok()?
        .parse::<f32>()
        .ok()?;
    Some(ExampleAutoExit(Timer::from_seconds(
        seconds.max(0.1),
        TimerMode::Once,
    )))
}

fn auto_exit_after_delay(
    time: Res<Time>,
    mut timer: ResMut<ExampleAutoExit>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}
