use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use web_sys::{
    Element, Event, EventTarget, HtmlElement, HtmlInputElement, InputEvent, MutationObserver,
    MutationObserverInit, MutationRecord, Node, NodeList,
};

const OTP_ROOT_SELECTOR: &str = "[data-otp-root]";
const OTP_INPUT_SELECTOR: &str = "input[data-otp-input]";
const OTP_SLOT_SELECTOR: &str = "[data-otp-slot]";
const OTP_CHAR_SELECTOR: &str = "[data-otp-char]";
const OTP_CARET_SELECTOR: &str = "[data-otp-caret]";
const OTP_INDEX_ATTR: &str = "data-otp-index";
const OTP_KEY_ATTR: &str = "data-otp-key";
const DEFAULT_MAX_LEN: usize = 6;

thread_local! {
    static MANAGER: RefCell<Option<OtpManager>> = const { RefCell::new(None) };
}

/// Wires up progressive-enhancement behaviour for every `InputOTP` root on the
/// page: it mirrors the hidden input's digits into the visible slots, drives the
/// active-slot highlight and blinking caret, restricts typing to ASCII digits,
/// and keeps focus on the input when a slot is clicked.
///
/// Idempotent and SSR-safe: the browser work runs inside an `Effect`, so calling
/// this during server render is a no-op. On the client it installs a single
/// shared manager that also watches the DOM for OTP roots added or removed
/// later, so it can be invoked from any OTP component without coordinating.
pub fn use_input_otp() {
    Effect::new(move |_| {
        MANAGER.with(|manager| {
            if manager.borrow().is_some() {
                return;
            }

            let mut otp_manager = OtpManager::new();
            otp_manager.init_all();
            otp_manager.observe_dom();
            *manager.borrow_mut() = Some(otp_manager);
        });
    });
}

/// Page-lifetime singleton that owns one [`OtpController`] per OTP root and a
/// `MutationObserver` that keeps that set in sync with the live DOM.
struct OtpManager {
    controllers: HashMap<String, OtpController>,
    next_key: u64,
    observer: Option<MutationObserver>,
    observer_callback: Option<Closure<dyn FnMut(js_sys::Array, MutationObserver)>>,
}

impl OtpManager {
    fn new() -> Self {
        Self {
            controllers: HashMap::new(),
            next_key: 0,
            observer: None,
            observer_callback: None,
        }
    }

    fn init_all(&mut self) {
        let Ok(nodes) = document().query_selector_all(OTP_ROOT_SELECTOR) else {
            return;
        };

        for root in elements(&nodes) {
            self.init_container(root);
        }
    }

    fn init_container(&mut self, root: Element) {
        let key = self.ensure_key(&root);
        if self.controllers.contains_key(&key) {
            return;
        }

        let Some(controller) = OtpController::new(root) else {
            return;
        };

        self.controllers.insert(key, controller);
    }

    fn remove_container(&mut self, root: &Element) {
        let Some(key) = root.get_attribute(OTP_KEY_ATTR) else {
            return;
        };

        self.controllers.remove(&key);
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "the MutationObserver callback owns the records array"
    )]
    fn handle_mutations(&mut self, records: js_sys::Array) {
        for record in records.iter() {
            let Ok(record) = record.dyn_into::<MutationRecord>() else {
                continue;
            };

            self.handle_node_list(&record.removed_nodes(), false);
            self.handle_node_list(&record.added_nodes(), true);
        }
    }

    fn handle_node_list(&mut self, nodes: &NodeList, added: bool) {
        for idx in 0..nodes.length() {
            let Some(node) = nodes.item(idx) else {
                continue;
            };
            self.handle_node(node, added);
        }
    }

    fn handle_node(&mut self, node: Node, added: bool) {
        let Ok(element) = node.dyn_into::<Element>() else {
            return;
        };

        for root in collect_roots(&element) {
            if added {
                self.init_container(root);
            } else {
                self.remove_container(&root);
            }
        }
    }

    fn ensure_key(&mut self, root: &Element) -> String {
        if let Some(key) = root.get_attribute(OTP_KEY_ATTR) {
            return key;
        }

        let key = format!("otp-{}", self.next_key);
        self.next_key += 1;
        _ = root.set_attribute(OTP_KEY_ATTR, &key);
        key
    }

    fn observe_dom(&mut self) {
        let Some(body) = document().body() else {
            return;
        };

        // The observer outlives this call, so it cannot borrow `self`; it reaches
        // the manager back through the shared `MANAGER` thread-local instead.
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                MANAGER.with(|manager| {
                    if let Some(manager) = manager.borrow_mut().as_mut() {
                        manager.handle_mutations(records);
                    }
                });
            },
        );

        let Ok(observer) = MutationObserver::new(callback.as_ref().unchecked_ref()) else {
            return;
        };

        let options = MutationObserverInit::new();
        options.set_child_list(true);
        options.set_subtree(true);

        if observer
            .observe_with_options(body.as_ref(), &options)
            .is_err()
        {
            return;
        }

        self.observer = Some(observer);
        self.observer_callback = Some(callback);
    }
}

impl Drop for OtpManager {
    fn drop(&mut self) {
        if let Some(observer) = &self.observer {
            observer.disconnect();
        }
    }
}

/// Behaviour bound to a single OTP root. Holds the DOM handles and the element
/// listeners alive; dropping it removes those listeners.
struct OtpController {
    _dom: Rc<OtpDom>,
    _listeners: Vec<Listener>,
}

impl OtpController {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "takes the root element handle obtained from a DOM query to build the controller"
    )]
    fn new(root: Element) -> Option<Self> {
        let input = find_input(&root)?;
        let dom = Rc::new(OtpDom::new(input, collect_slots(&root)));
        let listeners = register_listeners(&dom);
        update(&dom);
        Some(Self {
            _dom: dom,
            _listeners: listeners,
        })
    }
}

/// The resolved DOM of one OTP root: the hidden input, its slots ordered by
/// index, and the digit capacity read from the input's `maxlength`.
struct OtpDom {
    input: HtmlInputElement,
    slots: Vec<OtpSlot>,
    max_len: usize,
}

impl OtpDom {
    fn new(input: HtmlInputElement, mut slots: Vec<OtpSlot>) -> Self {
        slots.sort_by_key(|slot| slot.index);
        let max_len = input
            .get_attribute("maxlength")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_LEN);

        Self {
            input,
            slots,
            max_len,
        }
    }
}

/// One visible slot of an OTP root: its position, the slot element, and the
/// optional child elements that show the typed character and the blinking caret.
#[derive(Clone)]
struct OtpSlot {
    index: usize,
    slot: Element,
    char_el: Option<Element>,
    caret_el: Option<HtmlElement>,
}

/// An element-level event listener whose closure is dropped — and therefore
/// detached from the target — when this value is dropped.
struct Listener {
    target: EventTarget,
    event: &'static str,
    callback: Closure<dyn FnMut(Event)>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        _ = self.target.remove_event_listener_with_callback(
            self.event,
            self.callback.as_ref().unchecked_ref(),
        );
    }
}

fn elements(nodes: &NodeList) -> Vec<Element> {
    (0..nodes.length())
        .filter_map(|idx| nodes.item(idx))
        .filter_map(|node| node.dyn_into::<Element>().ok())
        .collect()
}

fn find_input(root: &Element) -> Option<HtmlInputElement> {
    root.query_selector(OTP_INPUT_SELECTOR)
        .ok()
        .flatten()
        .and_then(|input| input.dyn_into::<HtmlInputElement>().ok())
}

fn collect_slots(root: &Element) -> Vec<OtpSlot> {
    let Ok(nodes) = root.query_selector_all(OTP_SLOT_SELECTOR) else {
        return Vec::new();
    };

    elements(&nodes)
        .into_iter()
        .filter_map(|slot| {
            let index = slot
                .get_attribute(OTP_INDEX_ATTR)
                .and_then(|value| value.parse::<usize>().ok())?;
            let char_el = slot.query_selector(OTP_CHAR_SELECTOR).ok().flatten();
            let caret_el = slot
                .query_selector(OTP_CARET_SELECTOR)
                .ok()
                .flatten()
                .and_then(|caret| caret.dyn_into::<HtmlElement>().ok());

            Some(OtpSlot {
                index,
                slot,
                char_el,
                caret_el,
            })
        })
        .collect()
}

fn collect_roots(element: &Element) -> Vec<Element> {
    let mut roots = Vec::new();

    if element.matches(OTP_ROOT_SELECTOR).ok() == Some(true) {
        roots.push(element.clone());
    }

    if let Ok(nodes) = element.query_selector_all(OTP_ROOT_SELECTOR) {
        roots.extend(elements(&nodes));
    }

    roots
}

fn register_listeners(dom: &Rc<OtpDom>) -> Vec<Listener> {
    let mut listeners = register_slot_clicks(dom);
    listeners.extend(register_input_listeners(dom));
    listeners
}

fn register_slot_clicks(dom: &Rc<OtpDom>) -> Vec<Listener> {
    dom.slots
        .iter()
        .filter_map(|slot| {
            let dom = Rc::clone(dom);
            add_listener(slot.slot.clone().unchecked_into(), "click", move |_| {
                if dom.input.disabled() {
                    return;
                }

                _ = dom.input.focus();
            })
        })
        .collect()
}

fn register_input_listeners(dom: &Rc<OtpDom>) -> Vec<Listener> {
    let target: EventTarget = dom.input.clone().unchecked_into();

    [
        add_listener(target.clone(), "beforeinput", move |event| {
            filter_input(&event);
        }),
        {
            let dom = Rc::clone(dom);
            add_listener(target.clone(), "input", move |_| {
                update(&dom);
            })
        },
        {
            let dom = Rc::clone(dom);
            add_listener(target.clone(), "keydown", move |_| {
                defer({
                    let dom = Rc::clone(&dom);
                    move || update(&dom)
                });
            })
        },
        {
            let dom = Rc::clone(dom);
            add_listener(target.clone(), "focus", move |_| {
                defer({
                    let dom = Rc::clone(&dom);
                    move || {
                        move_cursor_to_end(&dom.input);
                        update(&dom);
                    }
                });
            })
        },
        {
            let dom = Rc::clone(dom);
            add_listener(target, "blur", move |_| {
                update(&dom);
            })
        },
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn add_listener<F>(target: EventTarget, event: &'static str, handler: F) -> Option<Listener>
where
    F: FnMut(Event) + 'static,
{
    let callback = Closure::<dyn FnMut(Event)>::new(handler);
    target
        .add_event_listener_with_callback(event, callback.as_ref().unchecked_ref())
        .ok()?;
    Some(Listener {
        target,
        event,
        callback,
    })
}

/// Cancels insertion of any non-digit character so the input can only ever hold
/// ASCII digits.
fn filter_input(event: &Event) {
    let Some(input_event) = event.dyn_ref::<InputEvent>() else {
        return;
    };

    if input_event.input_type() != "insertText" {
        return;
    }

    let Some(data) = input_event.data() else {
        return;
    };
    if data.chars().all(|ch| ch.is_ascii_digit()) {
        return;
    }

    input_event.prevent_default();
}

fn move_cursor_to_end(input: &HtmlInputElement) {
    let len = u32::try_from(input.value().chars().count()).unwrap_or(u32::MAX);
    _ = input.set_selection_range(len, len);
}

/// Runs `callback` on the next macrotask so the input's value and selection have
/// settled after the originating event. Only ever called from a client-side
/// event handler, so `window()` is available.
fn defer<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    let callback = Closure::once_into_js(callback);
    _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 0);
}

/// Reflects the input's current digits and caret position into every slot:
/// fills each slot's character, marks the active slot, and toggles its caret.
fn update(dom: &OtpDom) {
    let value: Vec<char> = dom.input.value().chars().collect();
    let input_element: Element = dom.input.clone().unchecked_into();
    let focused = document()
        .active_element()
        .is_some_and(|active| active == input_element);

    let selection = if focused {
        dom.input
            .selection_start()
            .ok()
            .flatten()
            .map_or(0, |position| position as usize)
    } else {
        usize::MAX
    };

    for slot in &dom.slots {
        let ch = value
            .get(slot.index)
            .copied()
            .unwrap_or_default()
            .to_string();
        let is_active = focused
            && (selection == slot.index
                || (selection >= value.len()
                    && slot.index == value.len()
                    && value.len() < dom.max_len));

        if let Some(char_el) = &slot.char_el {
            char_el.set_text_content(Some(&ch));
        }

        _ = slot
            .slot
            .set_attribute("data-active", if is_active { "true" } else { "false" });

        if let Some(caret_el) = &slot.caret_el {
            let display = if is_active && ch.is_empty() {
                "flex"
            } else {
                "none"
            };
            _ = caret_el.style().set_property("display", display);
        }
    }
}
