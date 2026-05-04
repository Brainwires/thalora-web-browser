//! Async iterators for `FileSystemDirectoryHandle`.
//!
//! Implements `keys()`, `values()`, `entries()` and `[Symbol.asyncIterator]`
//! per the WHATWG File System spec. The directory contents are snapshotted at
//! iterator creation time (one `read_dir` syscall) — `next()` returns
//! resolved Promises to match the rest of the engine's sync-resolution model.

use std::path::PathBuf;
use std::sync::Arc;

use boa_engine::{
    Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    builtins::{BuiltInBuilder, array::Array},
    js_string,
    object::{FunctionObjectBuilder, JsPromise},
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

use super::FileSystemDirectoryHandle;
use super::opfs_backend::{DirEntrySnapshot, OpfsBackend};

/// What this iterator yields per `next()`.
#[derive(Debug, Clone, Copy)]
pub enum IteratorKind {
    Keys,
    Values,
    Entries,
}

/// Internal state for a directory async iterator. Stored as a `JsData`
/// payload on the JS object returned to user code.
#[derive(Trace, Finalize, JsData)]
pub struct DirIteratorState {
    #[unsafe_ignore_trace]
    snapshot: Vec<DirEntrySnapshot>,
    #[unsafe_ignore_trace]
    backend: Arc<OpfsBackend>,
    #[unsafe_ignore_trace]
    parent_virtual_path: PathBuf,
    cursor: usize,
    #[unsafe_ignore_trace]
    kind: IteratorKind,
    is_opfs: bool,
}

impl std::fmt::Debug for DirIteratorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirIteratorState")
            .field("cursor", &self.cursor)
            .field("len", &self.snapshot.len())
            .field("kind", &self.kind)
            .finish()
    }
}

/// Build an async iterator JS object for the given directory handle.
pub fn build_iterator(
    dir: &FileSystemDirectoryHandle,
    kind: IteratorKind,
    context: &mut Context,
) -> JsResult<JsValue> {
    let backend = dir.backend.clone().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Async iteration is only supported on OPFS-rooted directory handles")
    })?;

    let snapshot = backend
        .read_dir(&dir.virtual_path)
        .map_err(|e| JsNativeError::error().with_message(format!("OPFS read_dir failed: {e}")))?;

    let state = DirIteratorState {
        snapshot,
        backend,
        parent_virtual_path: dir.virtual_path.clone(),
        cursor: 0,
        kind,
        is_opfs: dir.is_opfs,
    };

    let proto = context
        .intrinsics()
        .constructors()
        .object()
        .prototype();
    let iter_obj = JsObject::from_proto_and_data(Some(proto), state);

    let next_fn = BuiltInBuilder::callable(context.realm(), iterator_next)
        .name(js_string!("next"))
        .length(0)
        .build();
    iter_obj.set(js_string!("next"), next_fn, false, context)?;

    let return_fn = BuiltInBuilder::callable(context.realm(), iterator_return)
        .name(js_string!("return"))
        .length(0)
        .build();
    iter_obj.set(js_string!("return"), return_fn, false, context)?;

    let self_ref = FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(iterator_self),
    )
    .name(js_string!("[Symbol.asyncIterator]"))
    .length(0)
    .build();
    iter_obj.set(JsSymbol::async_iterator(), self_ref, false, context)?;

    Ok(iter_obj.into())
}

fn iterator_self(this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(this.clone())
}

fn iterator_next(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("next() called on non-object"))?;

    let (item, kind, parent_path, backend, is_opfs) = {
        let mut state = obj.downcast_mut::<DirIteratorState>().ok_or_else(|| {
            JsNativeError::typ().with_message("next() called on incompatible object")
        })?;

        if state.cursor >= state.snapshot.len() {
            return resolved_iter_result(JsValue::undefined(), true, context);
        }
        let item = state.snapshot[state.cursor].clone();
        state.cursor += 1;
        (
            item,
            state.kind,
            state.parent_virtual_path.clone(),
            state.backend.clone(),
            state.is_opfs,
        )
    };

    let value = match kind {
        IteratorKind::Keys => JsValue::from(boa_engine::JsString::from(item.name.clone())),
        IteratorKind::Values => {
            let handle = build_child_handle(&item, &parent_path, &backend, is_opfs, context)?;
            handle
        }
        IteratorKind::Entries => {
            let handle = build_child_handle(&item, &parent_path, &backend, is_opfs, context)?;
            let array = Array::array_create(2, None, context)?;
            array.set(
                0,
                JsValue::from(boa_engine::JsString::from(item.name.clone())),
                true,
                context,
            )?;
            array.set(1, handle, true, context)?;
            array.into()
        }
    };

    resolved_iter_result(value, false, context)
}

fn iterator_return(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    resolved_iter_result(JsValue::undefined(), true, context)
}

fn resolved_iter_result(value: JsValue, done: bool, context: &mut Context) -> JsResult<JsValue> {
    let result =
        boa_engine::builtins::iterable::create_iter_result_object(value, done, context);
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[result], context)?;
    Ok(JsValue::from(promise))
}

fn build_child_handle(
    entry: &DirEntrySnapshot,
    parent_virtual_path: &std::path::Path,
    backend: &Arc<OpfsBackend>,
    is_opfs: bool,
    context: &mut Context,
) -> JsResult<JsValue> {
    let child_path = parent_virtual_path.join(&entry.name);
    if entry.is_dir {
        let handle = FileSystemDirectoryHandle::new_opfs(
            entry.name.clone(),
            child_path,
            backend.clone(),
            is_opfs,
        );
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .file_system_directory_handle()
                .prototype(),
            handle,
        );
        Ok(obj.into())
    } else {
        let handle = super::FileSystemFileHandle::new_opfs(
            entry.name.clone(),
            child_path,
            backend.clone(),
            is_opfs,
        );
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .file_system_file_handle()
                .prototype(),
            handle,
        );
        Ok(obj.into())
    }
}

#[allow(dead_code)]
fn _ensure_args_used(_a: &[JsValue]) {
    // `_args` use suppressor for clippy
}
