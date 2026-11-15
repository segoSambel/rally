use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::{ActiveTheme, IndexPath, StyledExt};
use gpui_kit::{Context, Entity, Subscription, Window, div, prelude::*, px};

use rally_core::{BodyKind, KeyValueRow};

use super::key_value_editor::KeyValueEditor;

const KINDS: [(&str, BodyKind); 4] = [
    ("None", BodyKind::None),
    ("JSON", BodyKind::Json),
    ("Text", BodyKind::Text),
    ("Form (urlencoded)", BodyKind::Form),
];

/// Everything entered in the body tab.
pub struct BodyDraft {
    pub kind: BodyKind,
    pub text: String,
    pub form: Vec<KeyValueRow>,
}

type KindSelect = Vec<&'static str>;

pub struct BodyEditor {
    kind: BodyKind,
    kind_select: Entity<SelectState<KindSelect>>,
    /// Text for the JSON and Text kinds; only the highlighter differs.
    raw: Entity<EditorState>,
    form: Entity<KeyValueEditor>,
    _subscriptions: Vec<Subscription>,
}

impl BodyEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let names: KindSelect = KINDS.iter().map(|(name, _)| *name).collect();
        let kind_select =
            cx.new(|cx| SelectState::new(names, Some(IndexPath::default()), window, cx));
        let raw = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("text")
                .soft_wrap(true)
                .placeholder("Request body")
        });
        let form = cx.new(|cx| KeyValueEditor::new("Field", &[], window, cx));

        let subscriptions = vec![cx.subscribe_in(&kind_select, window, Self::on_kind_event)];
        Self {
            kind: BodyKind::None,
            kind_select,
            raw,
            form,
            _subscriptions: subscriptions,
        }
    }

    fn on_kind_event(
        &mut self,
        _select: &Entity<SelectState<KindSelect>>,
        event: &SelectEvent<KindSelect>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(name)) = event else {
            return;
        };
        if let Some((_, kind)) = KINDS.iter().find(|(label, _)| label == name) {
            self.set_kind(*kind, cx);
        }
    }

    fn set_kind(&mut self, kind: BodyKind, cx: &mut Context<Self>) {
        self.kind = kind;
        let language = if kind == BodyKind::Json {
            "json"
        } else {
            "text"
        };
        self.raw
            .update(cx, |raw, cx| raw.set_highlighter(language, cx));
        cx.notify();
    }

    pub fn kind(&self) -> BodyKind {
        self.kind
    }

    pub fn draft(&self, cx: &gpui_kit::App) -> BodyDraft {
        BodyDraft {
            kind: self.kind,
            text: self.raw.read(cx).value().to_string(),
            form: self.form.read(cx).rows(cx),
        }
    }

    /// Replaces everything entered. The kind is assigned directly because setting a select's
    /// value does not emit a selection event.
    pub fn set_draft(&mut self, draft: &BodyDraft, window: &mut Window, cx: &mut Context<Self>) {
        let (label, _) = KINDS
            .iter()
            .find(|(_, kind)| *kind == draft.kind)
            .unwrap_or(&KINDS[0]);
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(label, window, cx)
        });
        self.set_kind(draft.kind, cx);
        self.raw
            .update(cx, |raw, cx| raw.set_value(draft.text.clone(), window, cx));
        self.form
            .update(cx, |form, cx| form.set_rows(&draft.form, window, cx));
    }
}

impl Render for BodyEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.kind {
            BodyKind::None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("This request does not have a body.")
                .into_any_element(),
            BodyKind::Json | BodyKind::Text => Editor::new(&self.raw).h_full().into_any_element(),
            BodyKind::Form => self.form.clone().into_any_element(),
        };

        div()
            .size_full()
            .v_flex()
            .gap_3()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Body type"),
                    )
                    .child(div().w(px(200.)).child(Select::new(&self.kind_select))),
            )
            .child(div().flex_1().min_h_0().child(content))
    }
}
