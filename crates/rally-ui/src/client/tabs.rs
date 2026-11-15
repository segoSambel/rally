//! Request tabs. Each tab keeps its own request, response and pane selections. The editors
//! on screen always show the active tab; switching saves them into the old tab and restores
//! the new one.

use gpui_kit::{AppContext, Context, Entity, Subscription, Window};

use rally_core::{BodyKind, RequestDraft};

use super::model::ClientModel;
use super::view::ClientView;

pub(super) struct RequestTab {
    /// The request while the tab is in the background. For the active tab the live editors are
    /// the source of truth and this is stale.
    pub draft: RequestDraft,
    /// Per tab, so a request still in flight completes into the tab that sent it.
    pub model: Entity<ClientModel>,
    pub request_tab: usize,
    pub response_tab: usize,
    _subscriptions: Vec<Subscription>,
}

impl RequestTab {
    pub fn blank(model: Entity<ClientModel>, subscriptions: Vec<Subscription>) -> Self {
        Self {
            draft: RequestDraft {
                body_kind: BodyKind::None,
                ..Default::default()
            },
            model,
            request_tab: 0,
            response_tab: 0,
            _subscriptions: subscriptions,
        }
    }
}

impl ClientView {
    /// Hooks a tab's model into the view. The subscriptions live as long as the tab.
    pub(super) fn watch(
        model: &Entity<ClientModel>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        vec![
            cx.observe_in(model, window, Self::on_model_change),
            cx.subscribe(model, Self::on_request_completed),
        ]
    }

    /// Stores the live editors into the active tab.
    fn save_active(&mut self, cx: &Context<Self>) {
        let draft = self.snapshot(cx);
        let tab = &mut self.tabs[self.active_tab];
        tab.draft = draft;
        tab.request_tab = self.request_tab;
        tab.response_tab = self.response_tab;
    }

    /// Shows the active tab in the editors.
    fn load_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tab = &self.tabs[self.active_tab];
        let (draft, model) = (tab.draft.clone(), tab.model.clone());
        self.request_tab = tab.request_tab;
        self.response_tab = tab.response_tab;
        self.model = model;
        self.copied = false;
        self.restore(&draft, window, cx);
        self.sync_response(window, cx);
    }

    pub(super) fn new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.save_active(cx);
        let model = cx.new(|_| ClientModel::new());
        let subscriptions = Self::watch(&model, window, cx);
        self.tabs.push(RequestTab::blank(model, subscriptions));
        self.active_tab = self.tabs.len() - 1;
        self.load_active(window, cx);
    }

    pub(super) fn activate_tab(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if index == self.active_tab || index >= self.tabs.len() {
            return;
        }
        self.save_active(cx);
        self.active_tab = index;
        self.load_active(window, cx);
    }

    /// Closes a tab. The last tab cannot be closed. A request still in flight in a closed
    /// tab finishes in the background, but its result is dropped.
    pub(super) fn close_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.len() <= 1 || index >= self.tabs.len() {
            return;
        }
        let was_active = index == self.active_tab;
        self.tabs.remove(index);
        if index < self.active_tab {
            self.active_tab -= 1;
        } else if was_active {
            self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        }

        if was_active {
            self.load_active(window, cx);
        } else {
            cx.notify();
        }
    }
}
