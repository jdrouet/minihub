//! Toast notification system for showing transient messages.

use leptos::prelude::*;
use leptos::task::spawn_local;

/// Visual style of a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Error,
    Success,
}

impl ToastKind {
    /// CSS class suffix for the toast element.
    fn css_class(self) -> &'static str {
        match self {
            Self::Error => "toast-error",
            Self::Success => "toast-success",
        }
    }
}

/// A single toast message.
#[derive(Debug, Clone)]
pub struct ToastMessage {
    /// Unique id for keyed rendering.
    pub id: u32,
    /// The message body to display.
    pub text: String,
    /// Visual kind (error / success).
    pub kind: ToastKind,
}

/// Reactive context providing toast mutation methods.
#[derive(Clone)]
pub struct ToastProvider {
    set_toasts: WriteSignal<Vec<ToastMessage>>,
    next_id: ReadSignal<u32>,
    set_next_id: WriteSignal<u32>,
}

impl ToastProvider {
    /// Push a new error toast. It auto-dismisses after 5 seconds.
    pub fn push(&self, text: String) {
        self.push_with_kind(text, ToastKind::Error);
    }

    /// Push a new success toast. It auto-dismisses after 5 seconds.
    pub fn push_success(&self, text: String) {
        self.push_with_kind(text, ToastKind::Success);
    }

    fn push_with_kind(&self, text: String, kind: ToastKind) {
        let id = self.next_id.get_untracked();
        self.set_next_id.set(id + 1);

        self.set_toasts.update(|list| {
            list.push(ToastMessage { id, text, kind });
        });

        let set_toasts = self.set_toasts;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(5000).await;
            set_toasts.update(|list| {
                list.retain(|t| t.id != id);
            });
        });
    }

    /// Dismiss a toast immediately by id.
    pub fn dismiss(&self, id: u32) {
        self.set_toasts.update(|list| {
            list.retain(|t| t.id != id);
        });
    }
}

/// Access the toast provider from Leptos context.
///
/// Must be called within a component tree that has a [`ToastContainer`] ancestor.
pub fn use_toasts() -> ToastProvider {
    use_context::<ToastProvider>().expect("ToastProvider not found in context")
}

/// Container component that provides toast context and renders active toasts.
///
/// Place this once near the root of the component tree (e.g. inside `<App/>`).
#[component]
pub fn ToastContainer(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::<ToastMessage>::new());
    let (next_id, set_next_id) = signal(0_u32);

    let provider = ToastProvider {
        set_toasts,
        next_id,
        set_next_id,
    };

    provide_context(provider.clone());

    view! {
        {children()}
        <div class="toast-container">
            {move || {
                toasts
                    .get()
                    .into_iter()
                    .map(|toast| {
                        let id = toast.id;
                        let css = format!("toast {}", toast.kind.css_class());
                        let p = provider.clone();
                        view! {
                            <div class=css>
                                <button class="toast-dismiss" on:click=move |_| p.dismiss(id)>
                                    "\u{00D7}"
                                </button>
                                {toast.text}
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
