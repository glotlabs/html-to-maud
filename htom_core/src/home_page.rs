use crate::html_to_maud;
use crate::html_to_maud::{ClassStyle, IdStyle, Render};
use maud::html;
use polyester::browser;
use polyester::browser::app_effect;
use polyester::browser::effect::local_storage;
use polyester::browser::DomId;
use polyester::browser::Effect;
use polyester::browser::Effects;
use polyester::browser::ToDomId;
use polyester::page::Page;
use polyester::page::PageMarkup;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::max;

#[derive(strum_macros::Display, polyester_macro::ToDomId)]
#[strum(serialize_all = "kebab-case")]
enum Id {
    HtmlInput,
    MaudOutput,
    Settings,
    SettingsBackdrop,
    SettingsClose,
    RenderOptions,
    IdStyleOptions,
    ClassStyleOptions,
    KeyboardBindings,
}

pub struct HomePage {
    pub window_size: Option<browser::WindowSize>,
}

impl Page<Model, Msg, AppEffect> for HomePage {
    fn id(&self) -> DomId {
        DomId::new("html-to-maud")
    }

    fn init(&self) -> (Model, Effects<Msg, AppEffect>) {
        let html = r#"<div id="title" class="text-xl font-bold">Hello world</div>"#;
        let maud_config = html_to_maud::Config {
            render: Render::Auto,
            id_style: IdStyle::Full,
            class_style: ClassStyle::Full,
        };

        let model = Model {
            window_size: self.window_size.clone(),
            html: html.into(),
            maud: html_to_maud::html_to_maud(&html, &maud_config),
            show_settings: false,
            maud_config: maud_config,
            keyboard_bindings: KeyboardBindings::Default,
        };

        let effects = vec![load_settings_effect()];

        (model, effects)
    }

    fn subscriptions(&self, model: &Model) -> browser::Subscriptions<Msg, AppEffect> {
        let window_size_sub = browser::on_window_resize(Msg::WindowSizeChanged);

        if model.show_settings {
            vec![
                browser::on_click(&Id::SettingsBackdrop.to_dom_id(), Msg::HideSettings),
                browser::on_click(&Id::SettingsClose.to_dom_id(), Msg::HideSettings),
                browser::on_change(&Id::RenderOptions.to_dom_id(), Msg::RenderOptionChanged),
                browser::on_change(&Id::IdStyleOptions.to_dom_id(), Msg::IdStyleChanged),
                browser::on_change(&Id::ClassStyleOptions.to_dom_id(), Msg::ClassStyleChanged),
                browser::on_change(
                    &Id::KeyboardBindings.to_dom_id(),
                    Msg::KeyboardBindingsChanged,
                ),
                browser::on_keyup_document(browser::Key::Escape, Msg::EscapePressed),
                window_size_sub,
            ]
        } else {
            vec![
                browser::on_click_closest(&Id::Settings.to_dom_id(), Msg::ShowSettings),
                window_size_sub,
            ]
        }
    }

    fn update(&self, msg: &Msg, model: &mut Model) -> Result<Effects<Msg, AppEffect>, String> {
        match msg {
            Msg::GotSettings(value) => {
                let maybe_settings: Option<LocalStorageSettings> = value
                    .parse()
                    .map_err(|err| format!("Failed to parse settings: {}", err))?;

                if let Some(settings) = maybe_settings {
                    model.keyboard_bindings = settings.keyboard_bindings;
                    model.maud_config = settings.maud_config;
                    model.maud = html_to_maud::html_to_maud(&model.html, &model.maud_config);

                    Ok(vec![set_keyboard_handler_effect(model)])
                } else {
                    browser::no_effects()
                }
            }

            Msg::WindowSizeChanged(value) => {
                let window_size = value
                    .parse()
                    .map_err(|err| format!("Failed to parse window size: {}", err))?;

                model.window_size = Some(window_size);
                browser::no_effects()
            }

            Msg::HtmlChanged(html) => {
                model.html = html.into();
                model.maud = html_to_maud::html_to_maud(&html, &model.maud_config);
                browser::no_effects()
            }

            Msg::ShowSettings => {
                model.show_settings = true;
                browser::no_effects()
            }

            Msg::HideSettings => {
                model.show_settings = false;
                browser::no_effects()
            }

            Msg::EscapePressed => {
                model.show_settings = false;
                browser::no_effects()
            }

            Msg::RenderOptionChanged(value) => {
                let render = value
                    .parse()
                    .map_err(|err| format!("Failed to parse render option: {}", err))?;

                model.maud_config.render = render;
                model.maud = html_to_maud::html_to_maud(&model.html, &model.maud_config);
                Ok(vec![save_settings_effect(&model)])
            }

            Msg::IdStyleChanged(value) => {
                model.maud_config.id_style = value.parse().unwrap_or(IdStyle::Full);
                model.maud = html_to_maud::html_to_maud(&model.html, &model.maud_config);
                Ok(vec![save_settings_effect(&model)])
            }

            Msg::ClassStyleChanged(value) => {
                model.maud_config.class_style = value.parse().unwrap_or(ClassStyle::Full);
                model.maud = html_to_maud::html_to_maud(&model.html, &model.maud_config);
                Ok(vec![save_settings_effect(&model)])
            }

            Msg::KeyboardBindingsChanged(value) => {
                let keyboard_bindings = value
                    .parse()
                    .map_err(|err| format!("Failed to parse keyboard bindings: {}", err))?;

                model.keyboard_bindings = keyboard_bindings;

                Ok(vec![
                    set_keyboard_handler_effect(&model),
                    save_settings_effect(&model),
                ])
            }
        }
    }

    fn view(&self, model: &Model) -> PageMarkup {
        PageMarkup {
            head: view_head(),
            body: view_body(&self.id(), model),
        }
    }
}

fn view_head() -> maud::Markup {
    html! {
        title { "Html to Maud" }
        link rel="stylesheet" href="./app.css";
        script defer src="./vendor/ace/ace.js" {}
        script defer src="./vendor/ace/mode-html.js" {}
        script defer type="module" src="./home_page.js" {}
    }
}

fn view_body(page_id: &browser::DomId, model: &Model) -> maud::Markup {
    html! {
        div #(page_id) {
            nav class="bg-gray-800" {
                div class="px-2 sm:px-6 lg:px-8" {
                    div class="relative flex items-center justify-between h-16" {
                        div class="flex-1 flex sm:items-stretch sm:justify-start" {
                            div class="flex-shrink-0 flex items-center" {
                                h1 class="text-white text-2xl font-bold" { "Html to Maud" }
                            }
                        }
                        div class="sm:block sm:ml-6" {
                            div class="flex space-x-4" {
                                a href="https://github.com/prasmussen/html-to-maud" target="_blank" class="bg-gray-800 p-1 rounded-full text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white" type="button" {
                                    svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16" fill="currentColor" {
                                        path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" {}
                                    }
                                }
                                button #(Id::Settings) class="bg-gray-800 p-1 rounded-full text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white" type="button" {
                                    svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" {
                                        path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" {}
                                        path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" {}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            @if model.show_settings {
                (view_settings_modal(model))
            }

            @match &model.window_size {
                Some(window_size) => {
                    (view_editors(model, window_size))
                }

                None => {
                    (view_spinner())
                }
            }
        }
    }
}

fn view_editors(model: &Model, window_size: &browser::WindowSize) -> maud::Markup {
    let editor_height = max(i64::from(window_size.height) - 96, 500);
    let editor_style = format!("height: {}px;", editor_height);

    html! {
        div class="flex flex-col lg:flex-row pt-4" {
            div class="flex-1 pb-2 pl-4 pr-4 lg:pb-4 lg:pl-4 lg:pr-2" {
                div class="editor-container" style=(editor_style) {
                    div #(Id::HtmlInput) class="editor border border-gray-400 shadow" unmanaged { (model.html) }
                }
            }
            div class="flex-1 pt-2 pl-4 pr-4 pb-4 lg:pt-0 lg:pl-2 lg:pr-4" {
                div class="editor-container" style=(editor_style) {
                    textarea #(Id::MaudOutput) class="editor focus-border border border-gray-400 resize-none outline-none shadow" readonly { (model.maud) }
                }
            }
        }
    }
}

fn view_spinner() -> maud::Markup {
    html! {
        div class="spinner" {
            div class="rect1" {}
            div class="rect2" {}
            div class="rect3" {}
            div class="rect4" {}
            div class="rect5" {}
        }
    }
}

fn view_settings_modal(model: &Model) -> maud::Markup {
    html! {
        div class="relative z-10" aria-labelledby="modal-title" role="dialog" aria-modal="true" {
            div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" {}
            div class="fixed z-10 inset-0 overflow-y-auto" {
                div #(Id::SettingsBackdrop) class="flex items-end sm:items-center justify-center min-h-full p-4 text-center sm:p-0" {
                    div class="relative bg-white rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:max-w-sm sm:w-full sm:p-6" {
                        div {
                            div class="text-center" {
                                h3 class="text-lg leading-6 font-medium text-gray-900" {
                                    "Settings"
                                }
                            }

                            (view_dropdown("Render", Id::RenderOptions, &model.maud_config.render, vec![
                                ("Auto", &Render::Auto),
                                ("Full document", &Render::Full),
                                ("Only body", &Render::OnlyBody),
                            ]))

                            (view_dropdown("Id style", Id::IdStyleOptions, &model.maud_config.id_style, vec![
                                ("Full", &IdStyle::Full),
                                ("Short", &IdStyle::Short),
                                ("Short, implicit div", &IdStyle::ShortNoDiv),
                            ]))

                            (view_dropdown("Class style", Id::ClassStyleOptions, &model.maud_config.class_style, vec![
                                ("Full", &ClassStyle::Full),
                                ("Short", &ClassStyle::Short),
                                ("Short, implicit div", &ClassStyle::ShortNoDiv),
                            ]))

                            (view_dropdown("Keyboard bindings", Id::KeyboardBindings, &model.keyboard_bindings, vec![
                                ("Default", &KeyboardBindings::Default),
                                ("Vim", &KeyboardBindings::Vim),
                                ("Emacs", &KeyboardBindings::Emacs),
                            ]))
                        }

                        div class="mt-5 sm:mt-6" {
                            button #(Id::SettingsClose) class="inline-flex justify-center w-full rounded-md border border-transparent shadow-sm px-4 py-2 bg-indigo-600 text-base font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:text-sm" type="button" {
                                "Close"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn view_dropdown<V>(title: &str, id: Id, selected_value: V, options: Vec<(&str, V)>) -> maud::Markup
where
    V: PartialEq,
    V: Serialize,
{
    html! {
        div class="mt-4" {
            label class="block text-sm font-medium text-gray-700" for=(id) {
                (title)
            }
            select #(id) class="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md" {
                @for (name, value) in options {
                    option value=(json!(value)) selected[selected_value == value] {
                        (name)
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub window_size: Option<browser::WindowSize>,
    pub html: String,
    pub maud: String,
    pub show_settings: bool,
    pub maud_config: html_to_maud::Config,
    pub keyboard_bindings: KeyboardBindings,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Msg {
    HtmlChanged(String),
    WindowSizeChanged(browser::Value),
    ShowSettings,
    HideSettings,
    EscapePressed,
    RenderOptionChanged(browser::Value),
    IdStyleChanged(browser::Value),
    ClassStyleChanged(browser::Value),
    KeyboardBindingsChanged(browser::Value),
    GotSettings(browser::Value),
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "config")]
#[serde(rename_all = "camelCase")]
pub enum AppEffect {
    SetKeyboardHandler(String),
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum KeyboardBindings {
    Default,
    Vim,
    Emacs,
}

impl KeyboardBindings {
    fn ace_keyboard_handler(&self) -> String {
        match self {
            KeyboardBindings::Default => "".into(),
            KeyboardBindings::Vim => "ace/keyboard/vim".into(),
            KeyboardBindings::Emacs => "ace/keyboard/emacs".into(),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStorageSettings {
    pub maud_config: html_to_maud::Config,
    pub keyboard_bindings: KeyboardBindings,
}

fn load_settings_effect() -> Effect<Msg, AppEffect> {
    local_storage::get_item("settings", Msg::GotSettings)
}

fn save_settings_effect(model: &Model) -> Effect<Msg, AppEffect> {
    local_storage::set_item(
        "settings",
        LocalStorageSettings {
            keyboard_bindings: model.keyboard_bindings.clone(),
            maud_config: model.maud_config.clone(),
        },
    )
}

fn set_keyboard_handler_effect(model: &Model) -> Effect<Msg, AppEffect> {
    app_effect(AppEffect::SetKeyboardHandler(
        model.keyboard_bindings.ace_keyboard_handler(),
    ))
}
