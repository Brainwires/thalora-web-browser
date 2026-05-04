//! `FileSystemWritableFileStream` — the writer returned by `FileHandle.createWritable()`.
//!
//! Per spec, writes go to a *scratch* file; `close()` atomically renames it
//! over the target. This makes partially-written files invisible to readers
//! and lets `abort()` discard cleanly.
//!
//! Spec methods implemented: `write`, `seek`, `truncate`, `close`, `abort`.
//! `getWriter()` is intentionally not exposed — see PR notes.

use std::path::PathBuf;
use std::sync::Arc;

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::{JsPromise, builtins::JsArrayBuffer, builtins::JsTypedArray},
    realm::Realm,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};

use super::errors::{names, reject_with};
use super::opfs_backend::OpfsBackend;

#[derive(Trace, Finalize, JsData)]
pub struct FileSystemWritableFileStream {
    #[unsafe_ignore_trace]
    inner: parking_lot::Mutex<WritableInner>,
}

struct WritableInner {
    backend: Arc<OpfsBackend>,
    target_virtual: PathBuf,
    scratch_virtual: PathBuf,
    position: u64,
    closed: bool,
    aborted: bool,
}

impl std::fmt::Debug for FileSystemWritableFileStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSystemWritableFileStream").finish()
    }
}

impl FileSystemWritableFileStream {
    pub fn new(
        backend: Arc<OpfsBackend>,
        target_virtual: PathBuf,
        keep_existing_data: bool,
    ) -> std::io::Result<Self> {
        let scratch_virtual = scratch_path(&target_virtual);
        if keep_existing_data && backend.is_file(&target_virtual) {
            backend.copy_file(&target_virtual, &scratch_virtual)?;
        } else {
            backend.write_bytes(&scratch_virtual, &[])?;
        }
        Ok(Self {
            inner: parking_lot::Mutex::new(WritableInner {
                backend,
                target_virtual,
                scratch_virtual,
                position: 0,
                closed: false,
                aborted: false,
            }),
        })
    }
}

fn scratch_path(target: &std::path::Path) -> PathBuf {
    let suffix = format!(".crswap.{}", uuid::Uuid::new_v4().simple());
    let mut p = target.as_os_str().to_owned();
    p.push(suffix);
    PathBuf::from(p)
}

impl BuiltInObject for FileSystemWritableFileStream {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemWritableFileStream {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::write_js, js_string!("write"), 1)
            .method(Self::seek_js, js_string!("seek"), 1)
            .method(Self::truncate_js, js_string!("truncate"), 1)
            .method(Self::close_js, js_string!("close"), 0)
            .method(Self::abort_js, js_string!("abort"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemWritableFileStream {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;
    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &boa_engine::context::intrinsics::StandardConstructor =
        boa_engine::context::intrinsics::StandardConstructors::file_system_writable_file_stream;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemWritableFileStream is not directly constructible")
            .into())
    }
}

impl FileSystemWritableFileStream {
    fn write_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let chunk = args.get_or_undefined(0).clone();
        let result = Self::do_write(this, &chunk, context);
        finalize_result(result, context)
    }

    fn seek_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let pos = args.get_or_undefined(0).to_number(context)?;
        if pos < 0.0 || !pos.is_finite() {
            return reject_with(names::INVALID_STATE, "seek position is invalid", context);
        }
        let result = Self::do_seek(this, pos as u64);
        finalize_result(result, context)
    }

    fn truncate_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let size = args.get_or_undefined(0).to_number(context)?;
        if size < 0.0 || !size.is_finite() {
            return reject_with(names::INVALID_STATE, "truncate size is invalid", context);
        }
        let result = Self::do_truncate(this, size as u64);
        finalize_result(result, context)
    }

    fn close_js(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let result = Self::do_close(this);
        finalize_result(result, context)
    }

    fn abort_js(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let result = Self::do_abort(this);
        finalize_result(result, context)
    }

    fn do_write(
        this: &JsValue,
        chunk: &JsValue,
        context: &mut Context,
    ) -> Result<(), WritableError> {
        let bytes_or_command = parse_chunk(chunk, context)?;
        let obj = this
            .as_object()
            .ok_or_else(|| WritableError::Type("write() called on non-object".into()))?;
        let stream = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| WritableError::Type("write() called on incompatible object".into()))?;
        let mut inner = stream.inner.lock();
        if inner.closed || inner.aborted {
            return Err(WritableError::Dom(
                names::INVALID_STATE,
                "stream is closed".into(),
            ));
        }

        match bytes_or_command {
            Chunk::Data(data) => {
                write_at(&mut inner, &data)?;
            }
            Chunk::Write { position, data } => {
                if let Some(p) = position {
                    inner.position = p;
                }
                write_at(&mut inner, &data)?;
            }
            Chunk::Seek(p) => inner.position = p,
            Chunk::Truncate(s) => {
                truncate_inner(&inner, s)?;
                if inner.position > s {
                    inner.position = s;
                }
            }
        }
        Ok(())
    }

    fn do_seek(this: &JsValue, pos: u64) -> Result<(), WritableError> {
        let obj = this
            .as_object()
            .ok_or_else(|| WritableError::Type("seek() on non-object".into()))?;
        let stream = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| WritableError::Type("seek() on incompatible object".into()))?;
        let mut inner = stream.inner.lock();
        if inner.closed || inner.aborted {
            return Err(WritableError::Dom(
                names::INVALID_STATE,
                "stream is closed".into(),
            ));
        }
        inner.position = pos;
        Ok(())
    }

    fn do_truncate(this: &JsValue, size: u64) -> Result<(), WritableError> {
        let obj = this
            .as_object()
            .ok_or_else(|| WritableError::Type("truncate() on non-object".into()))?;
        let stream = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| WritableError::Type("truncate() on incompatible object".into()))?;
        let mut inner = stream.inner.lock();
        if inner.closed || inner.aborted {
            return Err(WritableError::Dom(
                names::INVALID_STATE,
                "stream is closed".into(),
            ));
        }
        truncate_inner(&inner, size)?;
        if inner.position > size {
            inner.position = size;
        }
        Ok(())
    }

    fn do_close(this: &JsValue) -> Result<(), WritableError> {
        let obj = this
            .as_object()
            .ok_or_else(|| WritableError::Type("close() on non-object".into()))?;
        let stream = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| WritableError::Type("close() on incompatible object".into()))?;
        let mut inner = stream.inner.lock();
        if inner.closed {
            return Ok(());
        }
        if inner.aborted {
            return Err(WritableError::Dom(
                names::INVALID_STATE,
                "stream was aborted".into(),
            ));
        }
        inner
            .backend
            .rename(&inner.scratch_virtual, &inner.target_virtual)
            .map_err(WritableError::Io)?;
        inner.closed = true;
        Ok(())
    }

    fn do_abort(this: &JsValue) -> Result<(), WritableError> {
        let obj = this
            .as_object()
            .ok_or_else(|| WritableError::Type("abort() on non-object".into()))?;
        let stream = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| WritableError::Type("abort() on incompatible object".into()))?;
        let mut inner = stream.inner.lock();
        if inner.closed || inner.aborted {
            return Ok(());
        }
        let _ = inner.backend.remove_file(&inner.scratch_virtual);
        inner.aborted = true;
        Ok(())
    }
}

fn write_at(inner: &mut WritableInner, data: &[u8]) -> Result<(), WritableError> {
    use std::io::{Seek, SeekFrom, Write};
    let mut file = inner
        .backend
        .open_file_rw(&inner.scratch_virtual, true)
        .map_err(WritableError::Io)?;
    file.seek(SeekFrom::Start(inner.position))
        .map_err(WritableError::Io)?;
    file.write_all(data).map_err(WritableError::Io)?;
    inner.position += data.len() as u64;
    Ok(())
}

fn truncate_inner(inner: &WritableInner, size: u64) -> Result<(), WritableError> {
    let file = inner
        .backend
        .open_file_rw(&inner.scratch_virtual, true)
        .map_err(WritableError::Io)?;
    file.set_len(size).map_err(WritableError::Io)?;
    Ok(())
}

enum Chunk {
    Data(Vec<u8>),
    Write { position: Option<u64>, data: Vec<u8> },
    Seek(u64),
    Truncate(u64),
}

enum WritableError {
    Type(String),
    Dom(&'static str, String),
    Io(std::io::Error),
}

fn parse_chunk(chunk: &JsValue, context: &mut Context) -> Result<Chunk, WritableError> {
    if let Some(s) = chunk.as_string() {
        return Ok(Chunk::Data(s.to_std_string_escaped().into_bytes()));
    }
    if let Some(obj) = chunk.as_object() {
        // Blob / File
        if let Some(blob) = obj.downcast_ref::<crate::file::blob::BlobData>() {
            return Ok(Chunk::Data((**blob.data()).clone()));
        }
        // ArrayBuffer
        if let Ok(arr_buf) = JsArrayBuffer::from_object(obj.clone()) {
            let bytes = arr_buf
                .data()
                .map(|d| d.to_vec())
                .unwrap_or_default();
            return Ok(Chunk::Data(bytes));
        }
        // TypedArray / DataView
        if let Ok(ta) = JsTypedArray::from_object(obj.clone()) {
            let buf_val = ta
                .buffer(context)
                .map_err(|e| WritableError::Type(format!("typed array buffer: {e:?}")))?;
            let buf_obj = buf_val
                .as_object()
                .ok_or_else(|| WritableError::Type("typed array has no buffer".into()))?;
            let arr_buf = JsArrayBuffer::from_object(buf_obj.clone())
                .map_err(|e| WritableError::Type(format!("typed array buffer downcast: {e:?}")))?;
            let off = ta
                .byte_offset(context)
                .map_err(|e| WritableError::Type(format!("byte_offset: {e:?}")))?;
            let len = ta
                .byte_length(context)
                .map_err(|e| WritableError::Type(format!("byte_length: {e:?}")))?;
            let bytes = arr_buf
                .data()
                .map(|d| d[off..off + len].to_vec())
                .unwrap_or_default();
            return Ok(Chunk::Data(bytes));
        }
        // Command-object {type, position?, size?, data?}
        if let Ok(type_val) = obj.get(js_string!("type"), context)
            && let Some(ty_str) = type_val.as_string()
        {
            let ty = ty_str.to_std_string_escaped();
            match ty.as_str() {
                "write" => {
                    let position = read_optional_u64(&obj, "position", context)?;
                    let data_val = obj
                        .get(js_string!("data"), context)
                        .map_err(|e| WritableError::Type(format!("data field: {e:?}")))?;
                    let data = match parse_chunk(&data_val, context)? {
                        Chunk::Data(d) => d,
                        _ => {
                            return Err(WritableError::Type(
                                "command write.data must be string/buffer/blob".into(),
                            ));
                        }
                    };
                    return Ok(Chunk::Write { position, data });
                }
                "seek" => {
                    let pos = read_optional_u64(&obj, "position", context)?
                        .ok_or_else(|| WritableError::Type("seek requires position".into()))?;
                    return Ok(Chunk::Seek(pos));
                }
                "truncate" => {
                    let size = read_optional_u64(&obj, "size", context)?
                        .ok_or_else(|| WritableError::Type("truncate requires size".into()))?;
                    return Ok(Chunk::Truncate(size));
                }
                _ => {
                    return Err(WritableError::Type(format!(
                        "unknown write command type: {ty}"
                    )));
                }
            }
        }
    }
    Err(WritableError::Type(
        "chunk must be a string, BufferSource, Blob, or write command".into(),
    ))
}

fn read_optional_u64(
    obj: &JsObject,
    key: &str,
    context: &mut Context,
) -> Result<Option<u64>, WritableError> {
    let v = obj
        .get(JsString::from(key), context)
        .map_err(|e| WritableError::Type(format!("{key}: {e:?}")))?;
    if v.is_undefined() || v.is_null() {
        return Ok(None);
    }
    let n = v
        .to_number(context)
        .map_err(|e| WritableError::Type(format!("{key} to_number: {e:?}")))?;
    if n < 0.0 || !n.is_finite() {
        return Err(WritableError::Type(format!("{key} must be a non-negative finite number")));
    }
    Ok(Some(n as u64))
}

fn finalize_result(
    result: Result<(), WritableError>,
    context: &mut Context,
) -> JsResult<JsValue> {
    match result {
        Ok(()) => {
            let (promise, resolvers) = JsPromise::new_pending(context);
            resolvers
                .resolve
                .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
            Ok(JsValue::from(promise))
        }
        Err(WritableError::Type(msg)) => Err(JsNativeError::typ().with_message(msg).into()),
        Err(WritableError::Dom(name, msg)) => reject_with(name, &msg, context),
        Err(WritableError::Io(e)) => {
            let name = super::errors::map_io_error(&e);
            reject_with(name, &format!("{e}"), context)
        }
    }
}
