use std::collections::HashMap;
use std::time::Instant;
use anyhow::{Result, anyhow};
use wasmtime::{Engine, Module, Store, Instance, Linker, Config};

/// Maximum number of simultaneously loaded WASM modules
pub const MAX_LOADED_MODULES: usize = 32;

/// Maximum WASM binary size (10MB)
pub const MAX_WASM_BINARY_SIZE: usize = 10 * 1024 * 1024;

/// Maximum memory read size (1MB)
pub const MAX_MEMORY_READ_SIZE: usize = 1024 * 1024;

/// Maximum fuel for execution (1 billion instructions)
pub const MAX_FUEL: u64 = 1_000_000_000;

/// Default fuel per execution call
pub const DEFAULT_FUEL: u64 = 1_000_000_000;

/// Maximum module ID length
pub const MAX_MODULE_ID_LENGTH: usize = 64;

/// Fuel metering state for a WASM store
pub struct FuelState {
    /// Total fuel consumed across all calls
    pub total_fuel_consumed: u64,
    /// Fuel limit for current execution
    pub fuel_limit: u64,
}

impl FuelState {
    pub fn new() -> Self {
        Self {
            total_fuel_consumed: 0,
            fuel_limit: DEFAULT_FUEL,
        }
    }
}

/// A loaded WASM module with its associated resources
pub struct LoadedModule {
    /// The compiled wasmtime Module
    pub module: Module,
    /// Optional instantiated instance (None if not instantiated)
    pub instance: Option<Instance>,
    /// The wasmtime Store with fuel metering
    pub store: Store<FuelState>,
    /// Raw binary of the module
    pub binary: Vec<u8>,
    /// Human-readable name
    pub name: String,
    /// When the module was loaded
    pub loaded_at: Instant,
}

/// Core state for WASM debug tools
pub struct WasmDebugState {
    /// Shared wasmtime engine with fuel consumption enabled
    engine: Engine,
    /// Map of module_id -> LoadedModule
    modules: HashMap<String, LoadedModule>,
}

impl WasmDebugState {
    /// Create a new WasmDebugState with a fuel-metering-enabled engine
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);

        let engine = Engine::new(&config)
            .map_err(|e| anyhow!("Failed to create wasmtime engine: {}", e))?;

        Ok(Self {
            engine,
            modules: HashMap::new(),
        })
    }

    /// Get a reference to the wasmtime engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get the number of loaded modules
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Check if a module ID exists
    pub fn has_module(&self, module_id: &str) -> bool {
        self.modules.contains_key(module_id)
    }

    /// Get a reference to a loaded module
    pub fn get_module(&self, module_id: &str) -> Option<&LoadedModule> {
        self.modules.get(module_id)
    }

    /// Get a mutable reference to a loaded module
    pub fn get_module_mut(&mut self, module_id: &str) -> Option<&mut LoadedModule> {
        self.modules.get_mut(module_id)
    }

    /// Insert a loaded module, returning error if at capacity
    pub fn insert_module(&mut self, module_id: String, module: LoadedModule) -> Result<()> {
        if self.modules.len() >= MAX_LOADED_MODULES {
            return Err(anyhow!(
                "Maximum number of loaded modules ({}) reached. Unload a module first.",
                MAX_LOADED_MODULES
            ));
        }
        self.modules.insert(module_id, module);
        Ok(())
    }

    /// Remove a module and return it
    pub fn remove_module(&mut self, module_id: &str) -> Option<LoadedModule> {
        self.modules.remove(module_id)
    }

    /// List all loaded modules with summary info
    pub fn list_modules(&self) -> Vec<ModuleSummary> {
        self.modules
            .iter()
            .map(|(id, m)| ModuleSummary {
                module_id: id.clone(),
                name: m.name.clone(),
                binary_size: m.binary.len(),
                is_instantiated: m.instance.is_some(),
                loaded_at_secs_ago: m.loaded_at.elapsed().as_secs(),
            })
            .collect()
    }

    /// Create a new Store with fuel metering for this engine
    pub fn new_store(&self) -> Store<FuelState> {
        let mut store = Store::new(&self.engine, FuelState::new());
        store.set_fuel(DEFAULT_FUEL).unwrap_or(());
        store.set_epoch_deadline(1);
        store.epoch_deadline_trap();
        store
    }
}

/// Summary information about a loaded module
#[derive(serde::Serialize)]
pub struct ModuleSummary {
    pub module_id: String,
    pub name: String,
    pub binary_size: usize,
    pub is_instantiated: bool,
    pub loaded_at_secs_ago: u64,
}

/// Validate a module ID string
pub fn validate_module_id(module_id: &str) -> Result<()> {
    if module_id.is_empty() {
        return Err(anyhow!("Module ID cannot be empty"));
    }
    if module_id.len() > MAX_MODULE_ID_LENGTH {
        return Err(anyhow!(
            "Module ID too long: {} chars (max {})",
            module_id.len(),
            MAX_MODULE_ID_LENGTH
        ));
    }
    if !module_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow!(
            "Module ID contains invalid characters: '{}'. Only alphanumeric, hyphens, and underscores are allowed",
            module_id
        ));
    }
    Ok(())
}
