// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Configuration trait for a CLI based on substrate

use crate::error::Result;
use crate::SubstrateCLI;
use app_dirs::{AppDataType, AppInfo};
use names::{Generator, Name};
use sc_service::config::{
	Configuration, DatabaseConfig, ExecutionStrategies, ExtTransport, KeystoreConfig,
	NetworkConfiguration, NodeKeyConfig, PrometheusConfig, PruningMode, Roles, TelemetryEndpoints,
	TransactionPoolOptions, WasmExecutionMethod,
};
use sc_service::ChainSpec;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

/// The maximum number of characters for a node name.
pub(crate) const NODE_NAME_MAX_LENGTH: usize = 32;

/// default sub directory to store network config
pub(crate) const DEFAULT_NETWORK_CONFIG_PATH: &'static str = "network";

/// A trait that allows converting an object to a Configuration
pub trait CliConfiguration: Sized {
	/// Get the base path of the configuration (if any)
	fn base_path(&self) -> Result<Option<&PathBuf>>;

	/// Returns `true` if the node is for development or not
	fn is_dev(&self) -> Result<bool> {
		Ok(false)
	}

	/// Get the roles
	fn roles(&self, _is_dev: bool) -> Result<Roles> {
		Ok(Roles::FULL)
	}

	/// Get the transaction pool options
	fn transaction_pool(&self) -> Result<TransactionPoolOptions> {
		Ok(Default::default())
	}

	/// Get the network configuration
	fn network_config(
		&self,
		_chain_spec: &Box<dyn ChainSpec>,
		_is_dev: bool,
		net_config_dir: &PathBuf,
		client_id: &str,
		node_name: &str,
		node_key: NodeKeyConfig,
	) -> Result<NetworkConfiguration> {
		Ok(NetworkConfiguration::new(
			node_name,
			client_id,
			node_key,
			net_config_dir,
		))
	}

	/// Get the keystore configuration
	fn keystore_config(&self, _base_path: &PathBuf) -> Result<KeystoreConfig> {
		Ok(KeystoreConfig::InMemory)
	}

	/// Get the database cache size (None for default)
	fn database_cache_size(&self) -> Result<Option<usize>> {
		Ok(Default::default())
	}

	/// Get the database configuration
	fn database_config(
		&self,
		base_path: &PathBuf,
		cache_size: Option<usize>,
	) -> Result<DatabaseConfig>;

	/// Get the state cache size
	fn state_cache_size(&self) -> Result<usize> {
		Ok(Default::default())
	}

	/// Get the state cache child ratio (if any)
	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		Ok(Default::default())
	}

	/// Get the pruning mode
	fn pruning(&self, _is_dev: bool, _roles: Roles) -> Result<PruningMode> {
		Ok(Default::default())
	}

	/// Get the chain spec
	fn chain_spec<C: SubstrateCLI>(&self) -> Result<Box<dyn ChainSpec>>;

	/// Get the name of the node
	fn node_name(&self) -> Result<String> {
		Ok(generate_node_name())
	}

	/// Get the WASM execution method
	fn wasm_method(&self) -> Result<WasmExecutionMethod> {
		Ok(Default::default())
	}

	/// Get the execution strategies
	fn execution_strategies(&self, _is_dev: bool) -> Result<ExecutionStrategies> {
		Ok(Default::default())
	}

	/// Get the RPC HTTP address (`None` if disabled)
	fn rpc_http(&self) -> Result<Option<SocketAddr>> {
		Ok(Default::default())
	}

	/// Get the RPC websocket address (`None` if disabled)
	fn rpc_ws(&self) -> Result<Option<SocketAddr>> {
		Ok(Default::default())
	}

	/// Get the RPC websockets maximum connections (`None` if unlimited)
	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		Ok(Default::default())
	}

	/// Get the RPC cors (`None` if disabled)
	fn rpc_cors(&self, _is_dev: bool) -> Result<Option<Vec<String>>> {
		Ok(Some(Vec::new()))
	}

	/// Get the prometheus configuration (`None` if disabled)
	fn prometheus_config(&self) -> Result<Option<PrometheusConfig>> {
		Ok(Default::default())
	}

	/// Get the telemetry endpoints (if any)
	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<TelemetryEndpoints>> {
		Ok(chain_spec.telemetry_endpoints().clone())
	}

	/// Get the telemetry external transport
	fn telemetry_external_transport(&self) -> Result<Option<ExtTransport>> {
		Ok(Default::default())
	}

	/// Get the default value for heap pages
	fn default_heap_pages(&self) -> Result<Option<u64>> {
		Ok(Default::default())
	}

	/// Returns `Ok(true)` if offchain worker should be used
	fn offchain_worker(&self, _roles: Roles) -> Result<bool> {
		Ok(Default::default())
	}

	/// Get sentry mode (i.e. act as an authority but **never** actively participate)
	fn sentry_mode(&self) -> Result<bool> {
		Ok(Default::default())
	}

	/// Returns `Ok(true)` if authoring should be forced
	fn force_authoring(&self) -> Result<bool> {
		Ok(Default::default())
	}

	/// Returns `Ok(true)` if grandpa should be disabled
	fn disable_grandpa(&self) -> Result<bool> {
		Ok(Default::default())
	}

	/// Get the development key seed from the current object
	fn dev_key_seed(&self, _is_dev: bool) -> Result<Option<String>> {
		Ok(Default::default())
	}

	/// Get the tracing targets from the current object (if any)
	fn tracing_targets(&self) -> Result<Option<String>> {
		Ok(Default::default())
	}

	/// Get the TracingReceiver value from the current object
	fn tracing_receiver(&self) -> Result<sc_tracing::TracingReceiver> {
		Ok(Default::default())
	}

	/// Get the node key from the current object
	fn node_key(&self, _net_config_dir: &PathBuf) -> Result<NodeKeyConfig> {
		Ok(Default::default())
	}

	/// Get maximum runtime instances
	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		Ok(Default::default())
	}

	/// Create a Configuration object from the current object
	fn create_configuration<C: SubstrateCLI>(
		&self,
		task_executor: Arc<dyn Fn(Pin<Box<dyn Future<Output = ()> + Send>>) + Send + Sync>,
	) -> Result<Configuration> {
		let chain_spec = self.chain_spec::<C>()?;
		let is_dev = self.is_dev()?;
		let default_config_dir = app_dirs::get_app_root(
			AppDataType::UserData,
			&AppInfo {
				name: C::get_executable_name(),
				author: C::get_author(),
			},
		)
		.expect("app directories exist on all supported platforms; qed");
		let config_dir = self
			.base_path()?
			.unwrap_or(&default_config_dir)
			.join("chains")
			.join(chain_spec.id());
		let net_config_dir = config_dir.join(DEFAULT_NETWORK_CONFIG_PATH);
		let client_id = C::client_id();
		// TODO: this parameter is really optional, shouldn't we leave it to None?
		let database_cache_size = Some(self.database_cache_size()?.unwrap_or(128));
		let node_key = self.node_key(&net_config_dir)?;
		let roles = self.roles(is_dev)?;
		let max_runtime_instances = self.max_runtime_instances()?.unwrap_or(8);

		Ok(Configuration {
			impl_name: C::get_impl_name(),
			impl_version: C::get_impl_version(),
			roles,
			task_executor,
			transaction_pool: self.transaction_pool()?,
			network: self.network_config(
				&chain_spec,
				is_dev,
				&net_config_dir,
				client_id.as_str(),
				self.node_name()?.as_str(),
				node_key,
			)?,
			keystore: self.keystore_config(&config_dir)?,
			database: self.database_config(&config_dir, database_cache_size)?,
			state_cache_size: self.state_cache_size()?,
			state_cache_child_ratio: self.state_cache_child_ratio()?,
			pruning: self.pruning(is_dev, roles)?,
			wasm_method: self.wasm_method()?,
			execution_strategies: self.execution_strategies(is_dev)?,
			rpc_http: self.rpc_http()?,
			rpc_ws: self.rpc_ws()?,
			rpc_ws_max_connections: self.rpc_ws_max_connections()?,
			rpc_cors: self.rpc_cors(is_dev)?,
			prometheus_config: self.prometheus_config()?,
			telemetry_endpoints: self.telemetry_endpoints(&chain_spec)?,
			telemetry_external_transport: self.telemetry_external_transport()?,
			default_heap_pages: self.default_heap_pages()?,
			offchain_worker: self.offchain_worker(roles)?,
			sentry_mode: self.sentry_mode()?,
			force_authoring: self.force_authoring()?,
			disable_grandpa: self.disable_grandpa()?,
			dev_key_seed: self.dev_key_seed(is_dev)?,
			tracing_targets: self.tracing_targets()?,
			tracing_receiver: self.tracing_receiver()?,
			chain_spec,
			max_runtime_instances,
		})
	}

	/// Initialize substrate. This must be done only once.
	///
	/// This method:
	///
	/// 1. Set the panic handler
	/// 2. Raise the FD limit
	/// 3. Initialize the logger
	fn init<C: SubstrateCLI>(&self) -> Result<()>;
}

/// Generate a valid random name for the node
pub fn generate_node_name() -> String {
	let result = loop {
		let node_name = Generator::with_naming(Name::Numbered).next().unwrap();
		let count = node_name.chars().count();

		if count < NODE_NAME_MAX_LENGTH {
			break node_name;
		}
	};

	result
}
