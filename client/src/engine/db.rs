use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbCursorWithValue, IdbDatabase, IdbFactory, IdbObjectStore, IdbOpenDbRequest, IdbRequest, IdbTransaction, IdbTransactionMode};

thread_local! {
    static CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

const DB_NAME: &str = "worlds_db";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "saves";

fn factory() -> Option<IdbFactory> {
    web_sys::window()?.indexed_db().ok()?
}

fn open_request() -> Option<IdbOpenDbRequest> {
    let f = factory()?;
    let r = f.open_with_u32(DB_NAME, DB_VERSION).ok()?;
    let r2 = r.clone();
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_ev: web_sys::Event| {
        if let Ok(v) = r2.result() {
            if let Ok(db) = v.dyn_into::<IdbDatabase>() {
                let _ = db.create_object_store(STORE_NAME);
            }
        }
    });
    r.set_onupgradeneeded(Some(cb.as_ref().unchecked_ref()));
    cb.forget();
    Some(r)
}

fn store_txn(db: &IdbDatabase, mode: IdbTransactionMode) -> Option<(IdbTransaction, IdbObjectStore)> {
    let tx = db.transaction_with_str_and_mode(STORE_NAME, mode).ok()?;
    let store = tx.object_store(STORE_NAME).ok()?;
    Some((tx, store))
}

fn request_as_promise(req: &IdbRequest) -> js_sys::Promise {
    let req_clone = req.clone();
    js_sys::Promise::new(&mut move |resolve: js_sys::Function, reject: js_sys::Function| {
        let resolve2 = resolve.clone();
        let reject2 = reject.clone();
        let req_for_success = req_clone.clone();
        let succ = Closure::<dyn FnMut(web_sys::Event)>::new(move |_ev: web_sys::Event| {
            if let Ok(val) = req_for_success.result() {
                let _ = resolve2.call1(&JsValue::null(), &val);
            } else {
                let _ = resolve2.call0(&JsValue::null());
            }
        });
        let fail = Closure::<dyn FnMut(web_sys::Event)>::new(move |_ev: web_sys::Event| {
            let _ = reject2.call0(&JsValue::null());
        });
        req_clone.set_onsuccess(Some(succ.as_ref().unchecked_ref()));
        req_clone.set_onerror(Some(fail.as_ref().unchecked_ref()));
        succ.forget();
        fail.forget();
    })
}

fn open_db() -> js_sys::Promise {
    let req = open_request().expect("IndexedDB not available");
    request_as_promise(&req)
}

/// Load all key-value pairs from IndexedDB into the in-memory cache.
fn load_cache_promise() -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve: js_sys::Function, _reject: js_sys::Function| {
        let resolve2 = resolve.clone();
        let open_req = match open_request() {
            Some(r) => r,
            _ => { let _ = resolve2.call0(&JsValue::null()); return; },
        };
        let open_req_cb = open_req.clone();
        let open_succ = Closure::<dyn FnMut(web_sys::Event)>::new(move |_ev: web_sys::Event| {
            let db: IdbDatabase = match open_req_cb.result().ok().and_then(|v| v.dyn_into().ok()) {
                Some(d) => d,
                _ => { let _ = resolve2.call0(&JsValue::null()); return; },
            };
            let (_tx, store) = match store_txn(&db, IdbTransactionMode::Readonly) {
                Some(s) => s,
                _ => { db.close(); let _ = resolve2.call0(&JsValue::null()); return; },
            };
            let cursor_req = match store.open_cursor() {
                Ok(c) => c,
                _ => { db.close(); let _ = resolve2.call0(&JsValue::null()); return; },
            };
            let cursor_req_clone = cursor_req.clone();

            let resolve3 = resolve2.clone();
            let cursor_cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_ev2: web_sys::Event| {
                if let Ok(val) = cursor_req_clone.result() {
                    if val.is_null() || val.is_undefined() {
                        db.close();
                        let _ = resolve3.call0(&JsValue::null());
                        return;
                    }
                    if let Some(cursor) = val.dyn_into::<IdbCursorWithValue>().ok() {
                        let k_opt = cursor.key().ok().and_then(|k| k.as_string());
                        let v_opt = cursor.value().ok().and_then(|v| v.as_string());
                        if let (Some(k), Some(v)) = (k_opt, v_opt) {
                            CACHE.with(|cache| {
                                cache.borrow_mut().insert(k, v);
                            });
                        }
                        let _ = cursor.continue_();
                    } else {
                        db.close();
                        let _ = resolve3.call0(&JsValue::null());
                    }
                } else {
                    db.close();
                    let _ = resolve3.call0(&JsValue::null());
                }
            });

            cursor_req.set_onsuccess(Some(cursor_cb.as_ref().unchecked_ref()));
            cursor_cb.forget();
        });
        open_req.set_onsuccess(Some(open_succ.as_ref().unchecked_ref()));
        open_succ.forget();
    })
}

/// Initialize: open DB, load all entries into cache.
pub async fn init_async() {
    let _db_fut = JsFuture::from(open_db()).await;
    let _cache_fut = JsFuture::from(load_cache_promise()).await;
}

/// Fire-and-forget async write: updates cache + IndexedDB
pub fn set_async(key: &str, value: &str) {
    CACHE.with(|cache| {
        cache.borrow_mut().insert(key.to_string(), value.to_string());
    });
    let key_owned = key.to_string();
    let value_owned = value.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let db_val = match JsFuture::from(open_db()).await {
            Ok(v) => v,
            _ => return,
        };
        let db: IdbDatabase = match db_val.dyn_into() {
            Ok(d) => d,
            _ => return,
        };
        if let Some((_tx, store)) = store_txn(&db, IdbTransactionMode::Readwrite) {
            let val = JsValue::from_str(&value_owned);
            let key = JsValue::from_str(&key_owned);
            let _ = store.put_with_key(&val, &key);
        }
        db.close();
    });
}

/// Fire-and-forget async delete: updates cache + IndexedDB
pub fn delete_async(key: &str) {
    CACHE.with(|cache| {
        cache.borrow_mut().remove(key);
    });
    let key_owned = key.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let db_val = match JsFuture::from(open_db()).await {
            Ok(v) => v,
            _ => return,
        };
        let db: IdbDatabase = match db_val.dyn_into() {
            Ok(d) => d,
            _ => return,
        };
        if let Some((_tx, store)) = store_txn(&db, IdbTransactionMode::Readwrite) {
            let _ = store.delete(&JsValue::from_str(&key_owned));
        }
        db.close();
    });
}

/// Synchronous read from in-memory cache
pub fn get(key: &str) -> Option<String> {
    CACHE.with(|cache| cache.borrow().get(key).cloned())
}

/// List all keys with a given prefix from the in-memory cache
pub fn keys_with_prefix(prefix: &str) -> Vec<String> {
    CACHE.with(|cache| {
        cache
            .borrow()
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k.clone())
            .collect()
    })
}

/// Migrate legacy localStorage data to IndexedDB (async, called once at startup)
pub async fn migrate_from_local_storage() {
    let window = match web_sys::window() {
        Some(w) => w,
        _ => return,
    };
    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        _ => return,
    };
    let db_val = match JsFuture::from(open_db()).await {
        Ok(v) => v,
        _ => return,
    };
    let db: IdbDatabase = match db_val.dyn_into() {
        Ok(d) => d,
        _ => return,
    };
    let mut migrated = false;
    for &legacy_key in &["worlds_autosave", "worlds_blocks"] {
        if let Ok(Some(val)) = storage.get_item(legacy_key) {
            if let Some((_tx, store)) = store_txn(&db, IdbTransactionMode::Readwrite) {
                let _ = store.put_with_key(&JsValue::from_str(&val), &JsValue::from_str(legacy_key));
                storage.remove_item(legacy_key).ok();
                CACHE.with(|cache| {
                    cache.borrow_mut().insert(legacy_key.to_string(), val);
                });
                migrated = true;
            }
        }
    }
    for i in 0u32..8u32 {
        let legacy_key = format!("worlds_save_{}", i);
        if let Ok(Some(val)) = storage.get_item(&legacy_key) {
            if let Some((_tx, store)) = store_txn(&db, IdbTransactionMode::Readwrite) {
                let _ = store.put_with_key(&JsValue::from_str(&val), &JsValue::from_str(&legacy_key));
                storage.remove_item(&legacy_key).ok();
                CACHE.with(|cache| {
                    cache.borrow_mut().insert(legacy_key, val);
                });
                migrated = true;
            }
        }
    }
    db.close();
    if migrated {
        web_sys::console::log_1(&"Migrated localStorage → IndexedDB".into());
    }
}
