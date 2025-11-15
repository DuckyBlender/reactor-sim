use bevy::{
    prelude::*,
    input_focus::tab_navigation::TabGroup, picking::hover::Hovered, prelude::*, ui::widget::Button, ui_widgets::{Activate, observe}
};

use crate::{
    GameState,
    simulation::{ControlSettings, EnvironmentState, GameOverReason}, ui::uranek::*,
};

pub mod indicators;
pub mod sliders;
mod game_over;
mod pause;
pub use pause::PauseState;
pub mod uranek;

#[derive(Component)]
pub struct UpgradeButton;
#[derive(Component)]
struct GameOverReasonText;

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
struct MoneyText;

#[derive(Component)]
struct Uranek;

#[derive(Component)]
struct UranekText;

#[derive(Component)]
struct UranekIdle;

#[derive(Component)]
struct UranekBubble;

#[derive(Resource)]
struct UranekState {
    last_blink_time: f32,
    blink_frame: u8,
    last_comment_time: f32,
    last_default_text_time: f32,
    talk_timeout: f32,
}

impl Default for UranekState {
    fn default() -> Self {
        Self {
            last_blink_time: 0.0,
            blink_frame: 0,
            last_comment_time: -999.0,
            last_default_text_time: 0.0,
            talk_timeout: 0.0,
        }
    }
}

#[derive(Resource)]
struct UranekAssets {
    spritesheet: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    idle_idx: usize,
    talk_idx: usize,
    hot_idx: usize,
struct PauseMenu;

#[derive(Component)]
struct ReturnToMenuButton;

#[derive(Resource, Default)]
pub struct PauseState {
    pub previous_state: Option<GameState>,
}

pub struct ReactorUiPlugin;

impl Plugin for ReactorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UranekState>()
        .add_systems(OnEnter(GameState::InGame), setup_game_ui)
        .add_systems(
            Update,
            (
                sliders::sync_slider_values,
                sliders::update_slider_visuals.after(sliders::sync_slider_values),
                sliders::update_slider_value_text,
                sliders::update_applied_value_text,
            ),
        )
        .add_systems(
            Update,
            (
                indicators::update_indicators,
                indicators::update_gauge_colors,
                indicators::handle_turbine_destroyed,
                indicators::rebuild_turbine_gauge_from_buyback,
                update_money_display,
                update_uranek_idle_animation,
                update_uranek_dialogue,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_plugins(GameOverPlugin)
        .add_plugins(PausePlugin);
    }
}

fn setup_game_ui(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    asset_server: Res<AssetServer>,
    mut uranek_state: ResMut<UranekState>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
) {
    *uranek_state = UranekState::default();
    uranek_state.last_default_text_time = time.elapsed_secs();

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");
    
    // Load spritesheet and create atlas layout
    let spritesheet_texture = asset_server.load("sprites/spritesheet.png");
    
    // Parse sprites.txt to define atlas layout
    // Format: talk,0,0,928,1120; idle,929,0,928,1120; wave,0,1121,928,1120; hot,929,1121,928,1120
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(1857, 2241));
    
    let talk_idx = layout.add_texture(URect::new(0, 0, 928, 1120));
    let idle_idx = layout.add_texture(URect::new(929, 0, 1857, 1120));
    layout.add_texture(URect::new(0, 1121, 928, 2241)); // wave
    let hot_idx = layout.add_texture(URect::new(929, 1121, 1857, 2241)); // hot
    
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Load and store Uranek assets once
    let uranek_assets = UranekAssets {
        spritesheet: spritesheet_texture.clone(),
        atlas_layout: texture_atlas_layout.clone(),
        idle_idx,
        talk_idx,
        hot_idx,
    };
    
    // Store sprite info for use before inserting resource
    let uranek_sprite_texture = uranek_assets.spritesheet.clone();
    let uranek_atlas_layout = uranek_assets.atlas_layout.clone();
    let uranek_idle_idx = uranek_assets.idle_idx;
    
    commands.insert_resource(uranek_assets);

    // Main UI root
    commands.spawn((
        DespawnOnExit(GameState::InGame),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(40.0)),
            row_gap: Val::Px(60.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        TabGroup::default(),
        Transform::default(),
        children![
            // Title
            (
                TextFont {
                    font: font.clone(),
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    ..default()
                },
            ),
            // Money display
            (
                Text::new("$0"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MoneyText,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    right: Val::Px(40.0),
                    ..default()
                },
            ),
            // Uranek companion (top-left corner)
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    align_items: AlignItems::End,
                    ..default()
                },
                children![
                    // Uranek sprite
                    (
                        ImageNode {
                            image: uranek_sprite_texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: uranek_atlas_layout,
                                index: uranek_idle_idx,
                            }),
                            ..default()
                        },
                        Node {
                            width: Val::Px(220.0),
                            height: Val::Px(220.0),
                            ..default()
                        },
                        Uranek,
                        UranekIdle,
                    ),
                    // Speech bubble (initially visible with default text)
                    (
                        Node {
                            width: Val::Px(420.0),
                            min_height: Val::Px(80.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
                        BorderRadius::all(Val::Px(12.0)),
                        BorderColor::all(Color::srgb(0.6, 0.9, 1.0)),
                        UranekBubble,
                        children![
                            (
                                Text::new("Uranek: Pilnujmy, żeby ten reaktor trzymał się w ryzach."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                UranekText,
                            ),
                        ],
                    ),
                ],
            ),
            (
                Button,
                Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(30.0),
                    max_width: Val::Px(300.0) ,
                    max_height: Val::Px(150.0), 
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    left: Val::Percent(80.0),
                    right: Val::Percent(1.0),
                    top: Val::Percent(35.0),
                    bottom: Val::Percent(50.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(12.0)),
                BorderColor::all(Color::srgba(0.83, 0.83, 0.83, 0.85)),
                BackgroundColor(Color::srgba(0.83, 0.83, 0.83, 0.85)),
                UpgradeButton,
                children![(
                    Text::new("Upgrade  67$"),
                    TextFont {
                        font: font.clone(),
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                )],
            ),
            // Bottom section with gauges and sliders
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    ..default()
                },
                Transform::default(),
                children![
                    // Left side - Gauges
                    indicators::gauge_grid(font.clone()),
                    // Right side - Sliders
                    sliders::slider_panel(
                        controls.reactivity_target,
                        controls.turbine_target,
                        font,
                        &asset_server
                    ),
                ],
            ),
        ],
    ));
}

fn update_money_display(
    environment: Res<EnvironmentState>,
    mut texts: Query<&mut Text, With<MoneyText>>,
) {
    if !environment.is_changed() {
        return;
    }

    for mut text in texts.iter_mut() {
        **text = format!("${:.0}", environment.money);
    }
}

fn update_uranek_idle_animation(
    time: Res<Time>,
    reactor: Res<ReactorState>,
    mut state: ResMut<UranekState>,
    mut image_query: Query<&mut ImageNode, With<UranekIdle>>,
    assets: Res<UranekAssets>,
) {
    let now = time.elapsed_secs();
    let reactor_ratio = reactor.temperature / REACTOR_TEMP_LIMIT;
    let reactor_pressure_ratio = reactor.pressure / REACTOR_PRESSURE_LIMIT;

    // Don't animate if reactor temperature OR pressure is in danger zone (hot sprite is shown)
    if reactor_ratio >= 0.80 || reactor_pressure_ratio >= 0.80 {
        return;
    }

    // Blink parameters: every few seconds a short blink for 1 second
    let interval = 8.0; // interval between blinks (eyes open)
    let blink_duration = 1.0; // time of 'closed' eyes

    if state.blink_frame == 1 {
        // Blink phase – after a second we return to the base frame
        if now - state.last_blink_time >= blink_duration {
            state.blink_frame = 0;
            for mut image_node in image_query.iter_mut() {
                if let Some(atlas) = &mut image_node.texture_atlas {
                    atlas.index = assets.idle_idx;
                }
            }
            // New reference point: from now we count the full interval to the next blink
            state.last_blink_time = now;
        }
        return;
    }

    // Eyes open – we check if the full interval to the next blink has passed
    if now - state.last_blink_time >= interval {
        state.last_blink_time = now;
        state.blink_frame = 1;
        for mut image_node in image_query.iter_mut() {
            if let Some(atlas) = &mut image_node.texture_atlas {
                atlas.index = assets.talk_idx;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_uranek_dialogue(
    time: Res<Time>,
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    environment: Res<EnvironmentState>,
    mut state: ResMut<UranekState>,
    mut text_query: Query<&mut Text, With<UranekText>>,
    mut bubble_query: Query<(&mut Node, &mut BackgroundColor), With<UranekBubble>>,
    mut sprite_query: Query<&mut ImageNode, With<Uranek>>,
    assets: Res<UranekAssets>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let now = time.elapsed_secs();

    // 1) Handle initial greeting lifetime
    if now - state.last_default_text_time > 3.0 && state.last_default_text_time > 0.0 {
        for mut text in text_query.iter_mut() {
            **text = String::new();
        }
        state.last_default_text_time = 0.0;
    }

    // 2) Handle continuous talk timeout (for warning/earnings lines)
    //    If talk_timeout expired and this is not the initial greeting, clear text.
    if state.talk_timeout > 0.0 && now > state.talk_timeout {
        for mut text in text_query.iter_mut() {
            **text = String::new();
        }
        state.talk_timeout = 0.0;
    }

    // 3) Determine if Uranek has something new to say
    const COMMENT_COOLDOWN: f32 = 6.0;
    let mut message: Option<&'static str> = None;

    let reactor_ratio = reactor.temperature / REACTOR_TEMP_LIMIT;
    let turbine_ratio = turbine.temperature / TURBINE_TEMP_LIMIT;
    let reactor_pressure_ratio = reactor.pressure / REACTOR_PRESSURE_LIMIT;

    // Temperature zones: Green (0-60%), Yellow (60-80%), Red (80-95%), Black (95-100%)

    if now - state.last_comment_time >= COMMENT_COOLDOWN {
        // Reactor temperature warnings (priority order: hottest first)
        if reactor_ratio >= 0.95 {
            // Black zone (95-100%) - critical meltdown imminent
            let messages = [
                "Uranek: REAKTOR ZARAZ SIĘ ROZTOPI! Obniż reaktywność NATYCHMIAST!",
                "Uranek: To koniec! Reaktor się topi! Wszystkie pręty TERAZ!",
                "Uranek: MELTDOWN! MELTDOWN! Obniż reaktywność do zera!",
                "Uranek: KRYTYCZNE! Reaktor przekracza limity! ZATRZYMAJ GO!",
                "Uranek: Widzę, że lubisz żyć niebezpiecznie... ZA BARDZO!",
                "Uranek: To nie jest konkurs na najgorętszy reaktor! Schłódź to!",
                "Uranek: Moja pensja nie obejmuje pracy w saunie! Obniż moc!",
                "Uranek: Jeśli to się roztopi, będę musiał pisać raport... NIE CHCĘ!",
                "Uranek: Reaktor robi się bardziej gorący niż moja była! ZATRZYMAJ!",
                "Uranek: To nie jest grill! Nie smażymy tu kiełbasek!",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if reactor_ratio >= 0.90 {
            // Deep red zone (90-95%)
            let messages = [
                "Uranek: Reaktor jest w strefie krytycznej! Obniż reaktywność!",
                "Uranek: Temperatura wchodzi w czarną strefę! Włóż pręty!",
                "Uranek: To już prawie koniec! Reaktor się topi!",
                "Uranek: Nie żartuj ze mną, to naprawdę niebezpieczne!",
                "Uranek: Chyba nie chcesz kończyć w gazecie jako 'operator roku'?",
                "Uranek: Reaktor robi się gorętszy niż moja kawa rano! To źle!",
                "Uranek: To nie jest konkurs piękności! Schłódź tego potwora!",
                "Uranek: Jeśli to wybuchnie, będę musiał tłumaczyć się szefowi...",
                "Uranek: Reaktor świeci się jak choinka, ale to nie święta!",
                "Uranek: Moja emerytura jest za 30 lat! Nie psuj mi planów!",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if reactor_ratio >= 0.80 {
            // Red zone (80-90%)
            let messages = [
                "Uranek: Reaktor jest głęboko w czerwonym. Włóż pręty, teraz.",
                "Uranek: Temperatura wchodzi w niebezpieczną strefę! Obniż reaktywność!",
                "Uranek: Reaktor się przegrzewa! Zmniejsz moc!",
                "Uranek: Za gorąco! Nie jestem upalem dla palących się problemów!",
                "Uranek: Czerwona strefa to nie dekoracja! Schłódź to!",
                "Uranek: Pamiętasz szkolenie BHP? Teraz przydałoby się!",
                "Uranek: Reaktor robi się gorętszy niż moje żarty! To naprawdę źle!",
                "Uranek: To nie jest sauna! Chyba że chcesz się zaparować...",
                "Uranek: Czerwony kolor nie oznacza 'idź szybciej'! Zwolnij!",
                "Uranek: Reaktor świeci się jak pomidor w słońcu! Schłódź go!",
                "Uranek: Moja cierpliwość też się kończy! Obniż reaktywność!",
                "Uranek: To nie jest wyścig! Kto pierwszy do meltdownu nie wygrywa!",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if reactor_ratio >= 0.60 {
            // Yellow zone (60-80%)
            let messages = [
                "Uranek: Reaktor wchodzi w żółtą strefę. Uważaj na temperaturę.",
                "Uranek: Temperatura rośnie. Może warto trochę zmniejszyć reaktywność?",
                "Uranek: Reaktor się rozgrzewa. Pilnuj wskaźników.",
                "Uranek: Zaczyna robić się ciepło. Trzymaj rękę na pulsie.",
                "Uranek: Żółta strefa - jeszcze bezpieczna, ale uważaj.",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if reactor_pressure_ratio >= 0.90 {
            // High pressure warning
            let messages = [
                "Uranek: Ciśnienie w reaktorze jest krytyczne! Obniż reaktywność!",
                "Uranek: Ciśnienie wchodzi w niebezpieczną strefę! Uwaga!",
                "Uranek: Za dużo ciśnienia! To nie garnek z bigos!",
                "Uranek: Reaktor zaraz wybuchnie jak szampan! Ale bez radości!",
                "Uranek: To nie jest konkurs na najwyższe ciśnienie! Obniż!",
                "Uranek: Ciśnienie rośnie szybciej niż moje ciśnienie krwi!",
                "Uranek: Reaktor puchnie jak balon! To nie jest dobry znak!",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if reactor_pressure_ratio >= 0.80 {
            // High pressure warning
            let messages = [
                "Uranek: Ciśnienie w reaktorze rośnie. Uważaj.",
                "Uranek: Manometr pokazuje za dużo. Kontroluj reaktywność.",
                "Uranek: Ciśnienie rośnie... jak moje obawy o tę zmianę!",
                "Uranek: Reaktor zaczyna się denerwować. Uspokój go!",
                "Uranek: To nie jest konkurs na najwyższe ciśnienie!",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if turbine_ratio >= 0.95 {
            // Turbine critical
            let messages = [
                "Uranek: Turbina wyje! Ochłodź ją, zanim wybuchnie!",
                "Uranek: Turbina w strefie krytycznej! Zmniejsz przepływ!",
                "Uranek: Turbina zaraz się rozleci! Mniej pary!",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if turbine_ratio >= 0.80 {
            // Turbine warning
            let messages = [
                "Uranek: Turbina jest w strefie zagrożenia. Zmniejsz przepływ.",
                "Uranek: Turbina się przegrzewa. Zmniejsz moc turbiny.",
                "Uranek: Turbina nie lubi takich temperatur. Przygaś trochę.",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if environment.money > 2000.0 {
            // Positive feedback - high earnings
            let messages = [
                "Uranek: Świetnie! Ta elektrownia drukuje pieniądze.",
                "Uranek: Doskonała robota! Wszystko działa jak należy.",
                "Uranek: Reaktor działa stabilnie. Trzymaj tak dalej!",
                "Uranek: Jesteś dobrym operatorem. Może nawet dostaniesz podwyżkę!",
                "Uranek: Widzę, że ten tutorial coś dał! Dobra robota.",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        } else if reactor_ratio < 0.40 && environment.money > 500.0 {
            // Low temperature, decent earnings
            let messages = [
                "Uranek: Reaktor działa spokojnie. Wszystko w porządku.",
                "Uranek: Stabilna praca, stabilna kasa. Tak to ma wyglądać.",
                "Uranek: W końcu mogę sobie spokojnie wypić kawę.",
            ];
            message = Some(messages[rand::rng().random_range(0..messages.len())]);
        }
    }

    // Update sprite based on reactor temperature or pressure zones (red/black = hot sprite)
    if let Ok(mut image_node) = sprite_query.single_mut()
        && let Some(atlas) = &mut image_node.texture_atlas {
        // Use hot sprite when reactor temperature OR pressure is in danger zone (>= 80%)
        let is_danger_zone = reactor_ratio >= 0.80 || reactor_pressure_ratio >= 0.80;
        if is_danger_zone {
            atlas.index = assets.hot_idx;
        } else if !is_danger_zone && atlas.index == assets.hot_idx {
            // Switch back to idle sprite when both temperature and pressure are safe
            atlas.index = assets.idle_idx;
        }
    }

    if let Some(msg) = message {
        // New line -> update text and set talk timeout
        for mut text in text_query.iter_mut() {
            **text = msg.to_string();
        }

        // Show speech bubble while talking
        if let Ok((mut node, mut bg)) = bubble_query.single_mut() {
            node.border = UiRect::all(Val::Px(2.0));
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85));
        }

        // Play random talking sound
        let random_num = rand::rng().random_range(1..=10);
        let sound_path = format!("sound/talking/talking_{:03}.mp3", random_num);
        let handle: Handle<AudioSource> = asset_server.load(sound_path);
        commands.spawn(AudioPlayer::new(handle));
        state.last_comment_time = now;

        // Bubble stays visible for this many seconds after the line
        state.talk_timeout = now + 4.0;
    }

    // 4) Final bubble visibility: show if there is any text, hide otherwise
    let mut has_any_text = false;
    for text in text_query.iter_mut() {
        if !text.to_string().is_empty() {
            has_any_text = true;
            break;
        }
    }

    if let Ok((mut node, mut bg)) = bubble_query.single_mut() {
        if has_any_text {
            node.border = UiRect::all(Val::Px(2.0));
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85));
        } else {
            node.border = UiRect::all(Val::Px(0.0));
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.0));
        }
    }
}
fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut pause_state: ResMut<PauseState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        pause_state.previous_state = Some(*current_state.get());
        next_state.set(GameState::Paused);
    }
}

fn handle_unpause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    pause_state: Res<PauseState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        let resume_state = pause_state.previous_state.unwrap_or(GameState::InGame);
        next_state.set(resume_state);
    }
}

fn setup_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pause_state: ResMut<PauseState>,
) {
    // Store the previous state if not already stored (fallback to InGame)
    if pause_state.previous_state.is_none() {
        pause_state.previous_state = Some(GameState::InGame);
    }
    
    commands.spawn((Camera2d, DespawnOnExit(GameState::Paused)));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    commands.spawn((
        DespawnOnExit(GameState::Paused),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Transform::default(),
        PauseMenu,
        children![
            (
                Text::new("PAUSED"),
                TextFont {
                    font: font.clone(),
                    font_size: 144.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                Transform::default(),
            ),
            (
                Text::new("Press ESC to resume"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
                Transform::default(),
            ),
            (
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(120.0),
                    border: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(16.0)),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Button,
                Hovered::default(),
                ReturnToMenuButton,
                observe(
                    |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                ),
                children![(
                    Text::new("Return to Menu"),
                    TextFont {
                        font,
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}

fn setup_game_over_ui(
    mut commands: Commands,
    game_over_reason: Res<GameOverReason>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((Camera2d, DespawnOnExit(GameState::GameOver)));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    let reason_text = match *game_over_reason {
        GameOverReason::ReactorExplosion => "REACTOR EXPLOSION",
        GameOverReason::ReactorMeltdown => "REACTOR MELTDOWN",
        GameOverReason::None => "Unknown cause",
    };

    // Game Over screen
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GameOverUI,
        children![
            (
                Text::new("GAME OVER"),
                TextFont {
                    font: font.clone(),
                    font_size: 144.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ),
            (
                Text::new(reason_text),
                TextFont {
                    font: font.clone(),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                GameOverReasonText,
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ),
            (
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(120.0),
                    border: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(16.0)),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Button,
                Hovered::default(),
                ReturnToMenuButton,
                observe(
                    |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                ),
                children![(
                    Text::new("Return to Menu"),
                    TextFont {
                        font,
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}
