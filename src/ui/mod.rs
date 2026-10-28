//! Модуль интерфейса: построение Bevy UI и маршрутизация ввода.
//!
//! Экран меню: чёрный фон, полупрозрачная панель с рамкой, заголовок,
//! пункты с выделением (цвет + подчёркивание), строка сообщения
//! и подсказка внизу.
//!
//! Ввод:
//!  * клавиатура — физические клавиши (`KeyCode`), раскладконезависимо;
//!  * мышь и тач — `Interaction` от Bevy UI (он обрабатывает и касания).

use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::text::{FontSource, FontSize, Underline};
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::config;
use crate::game::Game;
use crate::input::{self, Action};

/// Палитра интерфейса (перенесена из старого `gfx/frame.rs`).
mod colors {
    use bevy::prelude::Color;

    pub const TITLE: Color = Color::srgb(0.95, 0.96, 1.0);
    pub const ITEM: Color = Color::srgb(0.68, 0.71, 0.78);
    pub const ITEM_SELECTED: Color = Color::srgb(1.0, 0.82, 0.25);
    pub const MESSAGE: Color = Color::srgb(0.45, 0.9, 0.6);
    pub const HINT: Color = Color::srgb(0.55, 0.58, 0.68);
    pub const PANEL: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
    pub const BORDER: Color = Color::srgba(0.35, 0.45, 0.8, 0.8);
}

/// Маркер заголовка.
#[derive(Component)]
struct MenuTitle;

/// Маркер пункта меню с его индексом.
#[derive(Component)]
struct MenuItemTag(usize);

/// Маркер строки сообщения.
#[derive(Component)]
struct MessageText;

/// Обёртка игровой логики для Bevy: сама `Game` остаётся чистым Rust-кодом
/// (без derive-макросов Bevy) и полностью покрывается unit-тестами.
#[derive(Resource)]
struct GameState(Game);

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState(Game::new()))
            .add_systems(Startup, (setup_ui, hide_cursor))
            .add_systems(
                Update,
                (
                    make_window_visible,
                    handle_keyboard,
                    handle_pointer,
                    refresh_from_game,
                    exit_when_game_stops,
                ),
            );
    }
}

/// Строит всё дерево интерфейса меню.
// `Interaction` в Bevy 0.20 помечен как deprecated в пользу picking-стека
// (`Hovered`/`Pressed`), но именно его ведёт встроенная система `ui_focus_system`
// (и мышь, и тач) — для плоского UI без виджетов это рабочий путь.
#[allow(deprecated)]
fn setup_ui(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    // Камера обязательна: без неё Bevy UI не рендерится.
    commands.spawn(Camera2d);

    // Шрифт вшит в бинарь (см. config::FONT_DATA) — никакой загрузки ассетов.
    let font: Handle<Font> = fonts.add(Font::from_bytes(config::FONT_DATA.to_vec()));
    let text = |size: f32| TextFont {
        font: FontSource::Handle(font.clone()),
        font_size: FontSize::Px(size),
        ..default()
    };

    // Корневой узел: чёрный фон на весь экран, вертикальная колонка по центру.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            // Панель: полупрозрачный прямоугольник с рамкой.
            root.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: Val::Percent(48.0),
                    min_width: Val::Px(420.0),
                    max_width: Val::Px(760.0),
                    height: Val::Percent(72.0),
                    margin: UiRect::top(Val::Vh(10.0)),
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(colors::PANEL),
                Outline {
                    width: Val::Px(2.0),
                    offset: Val::ZERO,
                    color: colors::BORDER,
                },
            ))
            .with_children(|panel| {
                // Заголовок.
                panel.spawn((
                    Node {
                        margin: UiRect::top(Val::Vh(3.0)),
                        ..default()
                    },
                    Text::new(config::GAME_TITLE),
                    text(60.0),
                    TextColor(colors::TITLE),
                    MenuTitle,
                ));

                // Пункты меню: растянутая колонка, каждый пункт — по центру.
                panel
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_grow: 1.0,
                        row_gap: Val::Px(30.0),
                        ..default()
                    })
                    .with_children(|list| {
                        for (i, label) in config::MENU_ITEMS.iter().enumerate() {
                            list.spawn((
                                Node {
                                    padding: UiRect::horizontal(Val::Px(18.0)),
                                    ..default()
                                },
                                Text::new((*label).to_string()),
                                text(42.0),
                                TextColor(colors::ITEM),
                                Interaction::default(),
                                MenuItemTag(i),
                            ));
                        }
                    });

                // Сообщение о действии (изначально скрыто).
                panel.spawn((
                    Node {
                        margin: UiRect::top(Val::Px(24.0)),
                        ..default()
                    },
                    Text::new(String::new()),
                    text(28.0),
                    TextColor(colors::MESSAGE),
                    Visibility::Hidden,
                    MessageText,
                ));
            });

            // Подсказка по управлению: прижата к низу экрана.
            root.spawn((
                Node {
                    margin: UiRect {
                        top: Val::Auto,
                        bottom: Val::Vh(4.0),
                        ..default()
                    },
                    ..default()
                },
                Text::new(config::HINT),
                text(22.0),
                TextColor(colors::HINT),
            ));
        });
}

/// Прячет курсор: игра управляется с клавиатуры, а на тач-устройствах
/// курсора нет и так.
fn hide_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = false;
}

/// Показывает окно после первых кадров: пайплайн уже скомпилирован,
/// первый кадр отрисован — белой вспышки при запуске не будет.
fn make_window_visible(mut window: Single<&mut Window>, mut frames: Local<u32>) {
    *frames += 1;
    if *frames == 3 {
        window.visible = true;
    }
}

/// Клавиатура: физические клавиши -> действия игры.
fn handle_keyboard(mut state: ResMut<GameState>, keys: Res<ButtonInput<KeyCode>>) {
    for &key in &input::KEYS {
        if keys.just_pressed(key) {
            if let Some(action) = input::map_key(key) {
                state.0.handle_action(action);
            }
        }
    }
}

/// Мышь и тач: клик/тап по пункту — выбрать и подтвердить.
#[allow(deprecated)] // см. комментарий у setup_ui
fn handle_pointer(
    mut state: ResMut<GameState>,
    interactions: Query<(&Interaction, &MenuItemTag), Changed<Interaction>>,
) {
    for (interaction, tag) in &interactions {
        if *interaction == Interaction::Pressed {
            state.0.select(tag.0);
            state.0.handle_action(Action::Confirm);
        }
    }
}

/// Отражает состояние игры в интерфейсе (только при изменениях).
fn refresh_from_game(
    state: Res<GameState>,
    mut styled_items: Query<(&MenuItemTag, &mut TextColor, Entity)>,
    mut message_text: Query<&mut Text, With<MessageText>>,
    mut message_vis: Query<&mut Visibility, With<MessageText>>,
    mut commands: Commands,
) {
    if !state.is_changed() {
        return;
    }

    let view = state.0.view();

    // Выделение пунктов: цвет + подчёркивание у выбранного.
    for (tag, mut color, entity) in &mut styled_items {
        let selected = tag.0 == view.selected;
        color.0 = if selected {
            colors::ITEM_SELECTED
        } else {
            colors::ITEM
        };
        if selected {
            commands.entity(entity).insert(Underline);
        } else {
            commands.entity(entity).remove::<Underline>();
        }
    }

    // Сообщение о действии.
    if let Ok(mut text) = message_text.single_mut() {
        if let Some(msg) = &view.message {
            text.0 = msg.clone();
        } else {
            text.0 = String::new();
        }
    }
    if let Ok(mut vis) = message_vis.single_mut() {
        *vis = if view.message.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Завершает приложение, когда игровая логика решила выйти.
fn exit_when_game_stops(
    state: Res<GameState>,
    mut exited: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    if !state.0.is_running() && !*exited {
        *exited = true;
        crate::logging::info("выход из приложения по выбору пользователя");
        // В браузере закрыть вкладку нельзя — цикл просто остановится.
        exit.write(AppExit::Success);
    }
}
