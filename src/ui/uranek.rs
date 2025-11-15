use bevy::prelude::*;
use rand::Rng;

use crate::{
    simulation::{EnvironmentState, ReactorState, TurbineState, REACTOR_TEMP_LIMIT, REACTOR_PRESSURE_LIMIT, TURBINE_TEMP_LIMIT},
};

#[derive(Component)]
pub struct Uranek;

#[derive(Component)]
pub struct UranekText;

#[derive(Component)]
pub struct UranekIdle;

#[derive(Component)]
pub struct UranekBubble;

#[derive(Resource)]
pub struct UranekState {
    pub last_blink_time: f32,
    pub blink_frame: u8,
    pub last_comment_time: f32,
    pub last_default_text_time: f32,
    pub talk_timeout: f32,
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
pub struct UranekAssets {
    pub spritesheet: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub idle_idx: usize,
    pub talk_idx: usize,
    pub hot_idx: usize,
}

pub fn uranek_prefix(msg: &str) -> String {
    format!("Uranek: {}", msg)
}

pub fn update_uranek_idle_animation(
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
pub fn update_uranek_dialogue(
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
    let mut message: Option<String> = None;

    let reactor_ratio = reactor.temperature / REACTOR_TEMP_LIMIT;
    let turbine_ratio = turbine.temperature / TURBINE_TEMP_LIMIT;
    let reactor_pressure_ratio = reactor.pressure / REACTOR_PRESSURE_LIMIT;

    // Temperature zones: Green (0-60%), Yellow (60-80%), Red (80-95%), Black (95-100%)

    if now - state.last_comment_time >= COMMENT_COOLDOWN {
        // Reactor temperature warnings (priority order: hottest first)
        if reactor_ratio >= 0.95 {
            // Black zone (95-100%) - critical meltdown imminent
            let messages = [
                "REAKTOR ZARAZ SIĘ ROZTOPI! Obniż reaktywność NATYCHMIAST!",
                "To koniec! Reaktor się topi! Wszystkie pręty TERAZ!",
                "MELTDOWN! MELTDOWN! Obniż reaktywność do zera!",
                "KRYTYCZNE! Reaktor przekracza limity! ZATRZYMAJ GO!",
                "Widzę, że lubisz żyć niebezpiecznie... ZA BARDZO!",
                "To nie jest konkurs na najgorętszy reaktor! Schłódź to!",
                "Moja pensja nie obejmuje pracy w saunie! Obniż moc!",
                "Jeśli to się roztopi, będę musiał pisać raport... NIE CHCĘ!",
                "Reaktor robi się bardzo ciepły! To nie jest normalne!",
                "To nie jest grill! Nie smażymy tu kiełbasek!",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if reactor_ratio >= 0.90 {
            // Deep red zone (90-95%)
            let messages = [
                "Reaktor jest w strefie krytycznej! Obniż reaktywność!",
                "Temperatura wchodzi w czarną strefę! Włóż pręty!",
                "To już prawie koniec! Reaktor się topi!",
                "Nie żartuj ze mną, to naprawdę niebezpieczne!",
                "Chyba nie chcesz kończyć w gazecie jako 'operator roku'?",
                "Reaktor robi się gorętszy niż moja kawa rano! To źle!",
                "To nie jest konkurs piękności! Schłódź tego potwora!",
                "Jeśli to wybuchnie, będę musiał tłumaczyć się szefowi...",
                "Reaktor świeci się jak choinka, ale to nie święta!",
                "Moja emerytura jest za 30 lat! Nie psuj mi planów!",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if reactor_ratio >= 0.80 {
            // Red zone (80-90%)
            let messages = [
                "Reaktor jest głęboko w czerwonym. Włóż pręty, teraz.",
                "Temperatura wchodzi w niebezpieczną strefę! Obniż reaktywność!",
                "Reaktor się przegrzewa! Zmniejsz moc!",
                "Za gorąco! Nie jestem upalem dla palących się problemów!",
                "Czerwona strefa to nie dekoracja! Schłódź to!",
                "Pamiętasz szkolenie BHP? Teraz przydałoby się!",
                "Reaktor robi się gorętszy niż moje żarty! To naprawdę źle!",
                "To nie jest sauna! Chyba że chcesz się zaparować...",
                "Czerwony kolor nie oznacza 'idź szybciej'! Zwolnij!",
                "Reaktor świeci się jak pomidor w słońcu! Schłódź go!",
                "Moja cierpliwość też się kończy! Obniż reaktywność!",
                "To nie jest wyścig! Kto pierwszy do meltdownu nie wygrywa!",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if reactor_ratio >= 0.60 {
            // Yellow zone (60-80%)
            let messages = [
                "Reaktor wchodzi w żółtą strefę. Uważaj na temperaturę.",
                "Temperatura rośnie. Może warto trochę zmniejszyć reaktywność?",
                "Reaktor się rozgrzewa. Pilnuj wskaźników.",
                "Zaczyna robić się ciepło. Trzymaj rękę na pulsie.",
                "Żółta strefa - jeszcze bezpieczna, ale uważaj.",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if reactor_pressure_ratio >= 0.90 {
            // High pressure warning
            let messages = [
                "Ciśnienie w reaktorze jest krytyczne! Obniż reaktywność!",
                "Ciśnienie wchodzi w niebezpieczną strefę! Uwaga!",
                "Za dużo ciśnienia! To nie garnek z bigos!",
                "Reaktor zaraz wybuchnie jak szampan! Ale bez radości!",
                "To nie jest konkurs na najwyższe ciśnienie! Obniż!",
                "Ciśnienie rośnie szybciej niż moje ciśnienie krwi!",
                "Reaktor puchnie jak balon! To nie jest dobry znak!",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if reactor_pressure_ratio >= 0.80 {
            // High pressure warning
            let messages = [
                "Ciśnienie w reaktorze rośnie. Uważaj.",
                "Manometr pokazuje za dużo. Kontroluj reaktywność.",
                "Ciśnienie rośnie... jak moje obawy o tę zmianę!",
                "Reaktor zaczyna się denerwować. Uspokój go!",
                "To nie jest konkurs na najwyższe ciśnienie!",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if turbine_ratio >= 0.95 {
            // Turbine critical
            let messages = [
                "Turbina wyje! Ochłodź ją, zanim wybuchnie!",
                "Turbina w strefie krytycznej! Zmniejsz przepływ!",
                "Turbina zaraz się rozleci! Mniej pary!",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if turbine_ratio >= 0.80 {
            // Turbine warning
            let messages = [
                "Turbina jest w strefie zagrożenia. Zmniejsz przepływ.",
                "Turbina się przegrzewa. Zmniejsz moc turbiny.",
                "Turbina nie lubi takich temperatur. Przygaś trochę.",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if environment.money > 2000.0 {
            // Positive feedback - high earnings
            let messages = [
                "Świetnie! Ta elektrownia drukuje pieniądze.",
                "Doskonała robota! Wszystko działa jak należy.",
                "Reaktor działa stabilnie. Trzymaj tak dalej!",
                "Jesteś dobrym operatorem. Może nawet dostaniesz podwyżkę!",
                "Widzę, że ten tutorial coś dał! Dobra robota.",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
        } else if reactor_ratio < 0.40 && environment.money > 500.0 {
            // Low temperature, decent earnings
            let messages = [
                "Reaktor działa spokojnie. Wszystko w porządku.",
                "Stabilna praca, stabilna kasa. Tak to ma wyglądać.",
                "W końcu mogę sobie spokojnie wypić kawę.",
            ];
            message = Some(uranek_prefix(messages[rand::rng().random_range(0..messages.len())]));
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
            **text = msg.clone();
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