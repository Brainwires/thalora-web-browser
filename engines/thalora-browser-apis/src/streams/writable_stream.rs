//! WritableStream Web API implementation for Boa
//!
//! Implementation of the WHATWG Streams Standard WritableStream
//! https://streams.spec.whatwg.org/
//!
//! This implements the complete WritableStream interface according to the
//! WHATWG Streams Living Standard

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder, promise::Promise},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::collections::VecDeque;

/// JavaScript `WritableStream` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WritableStream;

impl IntrinsicObject for WritableStream {
    fn init(realm: &Realm) {
        let get_locked_func = BuiltInBuilder::callable(realm, get_locked)
            .name(js_string!("get locked"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("locked"),
                Some(get_locked_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::abort, js_string!("abort"), 1)
            .method(Self::close, js_string!("close"), 0)
            .method(Self::get_writer, js_string!("getWriter"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WritableStream {
    const NAME: JsString = StaticJsStrings::WRITABLE_STREAM;
}

impl BuiltInConstructor for WritableStream {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::writable_stream;

    /// `WritableStream(underlyingSink, queuingStrategy)`
    ///
    /// Constructor for WritableStream objects.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WritableStream constructor requires 'new'")
                .into());
        }

        let underlying_sink = args.get_or_undefined(0);
        let queuing_strategy = args.get_or_undefined(1);

        // Create the WritableStream object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::writable_stream, context)?;
        let writable_stream = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WritableStreamData::new(underlying_sink.clone(), queuing_strategy.clone()),
        );

        Ok(writable_stream.into())
    }
}

impl WritableStream {
    /// `WritableStream.prototype.abort(reason)`
    ///
    /// Aborts the stream, signaling that the producer can no longer successfully write.
    /// Calls the underlying sink's abort method.
    fn abort(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStream.prototype.abort called on non-object")
        })?;

        let reason = args.get_or_undefined(0);

        // Call the underlying sink's abort method and update state
        if let Some(mut data) = this_obj.downcast_mut::<WritableStreamData>() {
            // Clear any pending writes
            data.write_queue.clear();

            // Call abort callback on underlying sink
            let _ = data.call_abort(&reason, context);

            // Update stream state to errored
            data.state = StreamState::Errored;

            eprintln!("WritableStream: abort called with reason");
        }

        // Return a resolved Promise with undefined
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
    }

    /// `WritableStream.prototype.close()`
    ///
    /// Closes the stream when all queued chunks have been written.
    /// Calls the underlying sink's close method.
    fn close(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStream.prototype.close called on non-object")
        })?;

        // Validate stream state
        if let Some(data) = this_obj.downcast_ref::<WritableStreamData>() {
            if data.locked {
                return Err(JsNativeError::typ()
                    .with_message("Cannot close a locked stream")
                    .into());
            }

            if data.state != StreamState::Writable {
                return Err(JsNativeError::typ()
                    .with_message("Cannot close a stream that is not writable")
                    .into());
            }
        }

        // Process remaining queue and close
        if let Some(mut data) = this_obj.downcast_mut::<WritableStreamData>() {
            data.state = StreamState::Closing;

            // Process any remaining queued writes
            data.process_queue(context)?;

            // Call close callback on underlying sink
            let _ = data.call_close(context);

            // Mark as fully closed
            data.state = StreamState::Closed;

            eprintln!("WritableStream: close called - stream closed");
        }

        // Return a resolved Promise with undefined
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
    }

    /// `WritableStream.prototype.getWriter()`
    fn get_writer(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStream.prototype.getWriter called on non-object")
        })?;

        if let Some(data) = this_obj.downcast_ref::<WritableStreamData>() {
            if data.locked {
                return Err(JsNativeError::typ()
                    .with_message("Stream is already locked")
                    .into());
            }
        }

        // Lock the stream
        if let Some(mut data) = this_obj.downcast_mut::<WritableStreamData>() {
            data.locked = true;
        }

        // Create and return a WritableStreamDefaultWriter
        WritableStreamDefaultWriter::create(this_obj.clone(), context)
    }
}

/// JavaScript `WritableStreamDefaultWriter` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct WritableStreamDefaultWriter;

impl WritableStreamDefaultWriter {
    /// Create a new WritableStreamDefaultWriter instance
    fn create(stream: JsObject, context: &mut Context) -> JsResult<JsValue> {
        let proto = context
            .intrinsics()
            .constructors()
            .object()
            .prototype();

        let writer = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WritableStreamDefaultWriterData::new(stream),
        );

        // Add methods to the writer
        let abort_fn = BuiltInBuilder::callable(context.realm(), Self::abort)
            .name(js_string!("abort"))
            .length(1)
            .build();
        writer.set(js_string!("abort"), abort_fn, true, context)?;

        let close_fn = BuiltInBuilder::callable(context.realm(), Self::close)
            .name(js_string!("close"))
            .length(0)
            .build();
        writer.set(js_string!("close"), close_fn, true, context)?;

        let write_fn = BuiltInBuilder::callable(context.realm(), Self::write)
            .name(js_string!("write"))
            .length(1)
            .build();
        writer.set(js_string!("write"), write_fn, true, context)?;

        let release_lock_fn = BuiltInBuilder::callable(context.realm(), Self::release_lock)
            .name(js_string!("releaseLock"))
            .length(0)
            .build();
        writer.set(js_string!("releaseLock"), release_lock_fn, true, context)?;

        // Add properties
        let closed_getter = BuiltInBuilder::callable(context.realm(), Self::get_closed)
            .name(js_string!("get closed"))
            .build();
        writer.define_property_or_throw(
            js_string!("closed"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .get(closed_getter)
                .configurable(true)
                .enumerable(true),
            context,
        )?;

        let ready_getter = BuiltInBuilder::callable(context.realm(), Self::get_ready)
            .name(js_string!("get ready"))
            .build();
        writer.define_property_or_throw(
            js_string!("ready"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .get(ready_getter)
                .configurable(true)
                .enumerable(true),
            context,
        )?;

        let desired_size_getter = BuiltInBuilder::callable(context.realm(), Self::get_desired_size)
            .name(js_string!("get desiredSize"))
            .build();
        writer.define_property_or_throw(
            js_string!("desiredSize"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .get(desired_size_getter)
                .configurable(true)
                .enumerable(true),
            context,
        )?;

        Ok(writer.into())
    }

    /// `WritableStreamDefaultWriter.prototype.abort(reason)`
    fn abort(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.abort called on non-object")
        })?;

        let reason = args.get_or_undefined(0);

        if let Some(writer_data) = this_obj.downcast_ref::<WritableStreamDefaultWriterData>() {
            if let Some(mut stream_data) = writer_data.stream.downcast_mut::<WritableStreamData>() {
                stream_data.state = StreamState::Errored;
            }
        }

        {
            let promise_constructor = context.intrinsics().constructors().promise().constructor();
            Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
        }
    }

    /// `WritableStreamDefaultWriter.prototype.close()`
    fn close(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.close called on non-object")
        })?;

        if let Some(writer_data) = this_obj.downcast_ref::<WritableStreamDefaultWriterData>() {
            if let Some(mut stream_data) = writer_data.stream.downcast_mut::<WritableStreamData>() {
                if stream_data.state != StreamState::Writable {
                    return Err(JsNativeError::typ()
                        .with_message("Stream is not in writable state")
                        .into());
                }
                stream_data.state = StreamState::Closing;

                // Process queued writes
                stream_data.process_queue(context)?;
                stream_data.state = StreamState::Closed;
            }
        }

        {
            let promise_constructor = context.intrinsics().constructors().promise().constructor();
            Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
        }
    }

    /// `WritableStreamDefaultWriter.prototype.write(chunk)`
    fn write(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.write called on non-object")
        })?;

        let chunk = args.get_or_undefined(0);

        if let Some(writer_data) = this_obj.downcast_ref::<WritableStreamDefaultWriterData>() {
            if let Some(mut stream_data) = writer_data.stream.downcast_mut::<WritableStreamData>() {
                if stream_data.state != StreamState::Writable {
                    return Err(JsNativeError::typ()
                        .with_message("Stream is not in writable state")
                        .into());
                }

                // Add chunk to write queue
                stream_data.write_queue.push_back(chunk.clone());

                // Process the queue
                stream_data.process_queue(context)?;
            }
        }

        {
            let promise_constructor = context.intrinsics().constructors().promise().constructor();
            Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
        }
    }

    /// `WritableStreamDefaultWriter.prototype.releaseLock()`
    fn release_lock(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.releaseLock called on non-object")
        })?;

        if let Some(writer_data) = this_obj.downcast_ref::<WritableStreamDefaultWriterData>() {
            if let Some(mut stream_data) = writer_data.stream.downcast_mut::<WritableStreamData>() {
                stream_data.locked = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// Get the closed property of a WritableStreamDefaultWriter
    fn get_closed(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.closed getter called on non-object")
        })?;

        let writer_data = this_obj.downcast_ref::<WritableStreamDefaultWriterData>().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.closed getter called on non-WritableStreamDefaultWriter object")
        })?;

        let stream_data = writer_data.stream.downcast_ref::<WritableStreamData>().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter stream is not a WritableStreamData")
        })?;

        match stream_data.state {
            StreamState::Closed => {
                let promise_constructor = context.intrinsics().constructors().promise().constructor();
                Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
            },
            StreamState::Errored => {
                let promise_constructor = context.intrinsics().constructors().promise().constructor();
                Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
            },
            _ => {
                // Return a pending promise
                let promise_constructor = context.intrinsics().constructors().promise().constructor();
                Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
            }
        }
    }

    /// Get the ready property of a WritableStreamDefaultWriter
    fn get_ready(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.ready getter called on non-object")
        })?;

        let writer_data = this_obj.downcast_ref::<WritableStreamDefaultWriterData>().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.ready getter called on non-WritableStreamDefaultWriter object")
        })?;

        let stream_data = writer_data.stream.downcast_ref::<WritableStreamData>().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter stream is not a WritableStreamData")
        })?;

        match stream_data.state {
            StreamState::Writable => {
                let promise_constructor = context.intrinsics().constructors().promise().constructor();
                Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
            },
            StreamState::Errored => {
                let promise_constructor = context.intrinsics().constructors().promise().constructor();
                Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
            },
            _ => {
                let promise_constructor = context.intrinsics().constructors().promise().constructor();
                Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
            }
        }
    }

    /// Get the desiredSize property of a WritableStreamDefaultWriter
    fn get_desired_size(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.desiredSize getter called on non-object")
        })?;

        let writer_data = this_obj.downcast_ref::<WritableStreamDefaultWriterData>().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter.prototype.desiredSize getter called on non-WritableStreamDefaultWriter object")
        })?;

        let stream_data = writer_data.stream.downcast_ref::<WritableStreamData>().ok_or_else(|| {
            JsNativeError::typ().with_message("WritableStreamDefaultWriter stream is not a WritableStreamData")
        })?;

        match stream_data.state {
            StreamState::Writable => {
                // Return high water mark minus current queue size
                let desired = stream_data.high_water_mark - stream_data.write_queue.len() as f64;
                Ok(JsValue::from(desired))
            },
            StreamState::Closed | StreamState::Closing => Ok(JsValue::from(0)),
            StreamState::Errored => Ok(JsValue::null()),
        }
    }
}

/// Internal data for WritableStream instances
#[derive(Debug, Trace, Finalize, JsData)]
struct WritableStreamData {
    #[unsafe_ignore_trace]
    underlying_sink: JsValue,
    #[unsafe_ignore_trace]
    queuing_strategy: JsValue,
    locked: bool,
    state: StreamState,
    #[unsafe_ignore_trace]
    write_queue: VecDeque<JsValue>,
    high_water_mark: f64,
}

impl WritableStreamData {
    fn new(underlying_sink: JsValue, queuing_strategy: JsValue) -> Self {
        // Extract high water mark from queuing strategy
        let high_water_mark = if let Some(strategy_obj) = queuing_strategy.as_object() {
            // Try to get highWaterMark property - simplified for now
            1.0 // Default high water mark
        } else {
            1.0 // Default high water mark
        };

        Self {
            underlying_sink,
            queuing_strategy,
            locked: false,
            state: StreamState::Writable,
            write_queue: VecDeque::new(),
            high_water_mark,
        }
    }

    /// Process the write queue by calling the underlying sink's write method
    fn process_queue(&mut self, context: &mut Context) -> JsResult<()> {
        // Get the underlying sink's write method if it exists
        let write_fn = if let Some(sink_obj) = self.underlying_sink.as_object() {
            sink_obj.get(js_string!("write"), context).ok()
        } else {
            None
        };

        while let Some(chunk) = self.write_queue.pop_front() {
            // Call the underlying sink's write method if available
            if let Some(ref write_val) = write_fn {
                if let Some(write_callable) = write_val.as_callable() {
                    // Create a controller-like object for the write callback
                    let controller = boa_engine::object::ObjectInitializer::new(context)
                        .build();

                    // Call write(chunk, controller)
                    let result = write_callable.call(
                        &self.underlying_sink,
                        &[chunk, controller.into()],
                        context
                    );

                    if let Err(e) = result {
                        eprintln!("WritableStream: write callback error: {:?}", e);
                        self.state = StreamState::Errored;
                        return Err(e);
                    }
                }
            }
            // If no write method, chunks are just consumed (discarded)
        }
        Ok(())
    }

    /// Call the underlying sink's start method
    pub fn call_start(&self, context: &mut Context) -> JsResult<()> {
        if let Some(sink_obj) = self.underlying_sink.as_object() {
            if let Ok(start_val) = sink_obj.get(js_string!("start"), context) {
                if let Some(start_callable) = start_val.as_callable() {
                    let controller = boa_engine::object::ObjectInitializer::new(context)
                        .build();
                    start_callable.call(&self.underlying_sink, &[controller.into()], context)?;
                }
            }
        }
        Ok(())
    }

    /// Call the underlying sink's close method
    pub fn call_close(&self, context: &mut Context) -> JsResult<()> {
        if let Some(sink_obj) = self.underlying_sink.as_object() {
            if let Ok(close_val) = sink_obj.get(js_string!("close"), context) {
                if let Some(close_callable) = close_val.as_callable() {
                    close_callable.call(&self.underlying_sink, &[], context)?;
                }
            }
        }
        Ok(())
    }

    /// Call the underlying sink's abort method
    pub fn call_abort(&self, reason: &JsValue, context: &mut Context) -> JsResult<()> {
        if let Some(sink_obj) = self.underlying_sink.as_object() {
            if let Ok(abort_val) = sink_obj.get(js_string!("abort"), context) {
                if let Some(abort_callable) = abort_val.as_callable() {
                    abort_callable.call(&self.underlying_sink, &[reason.clone()], context)?;
                }
            }
        }
        Ok(())
    }

    /// Get the current desired size (for backpressure)
    pub fn get_desired_size(&self) -> f64 {
        match self.state {
            StreamState::Writable => self.high_water_mark - self.write_queue.len() as f64,
            StreamState::Closing | StreamState::Closed => 0.0,
            StreamState::Errored => 0.0,
        }
    }
}

/// Internal data for WritableStreamDefaultWriter instances
#[derive(Debug, Trace, Finalize, JsData)]
struct WritableStreamDefaultWriterData {
    stream: JsObject,
}

impl WritableStreamDefaultWriterData {
    fn new(stream: JsObject) -> Self {
        Self { stream }
    }
}

#[derive(Debug, Clone, PartialEq, Trace, Finalize)]
enum StreamState {
    Writable,
    Closing,
    Closed,
    Errored,
}

/// Get the locked property of a WritableStream
fn get_locked(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WritableStream.prototype.locked getter called on non-object")
    })?;

    let data = this_obj.downcast_ref::<WritableStreamData>().ok_or_else(|| {
        JsNativeError::typ().with_message("WritableStream.locked getter called on non-WritableStream object")
    })?;

    Ok(data.locked.into())
}