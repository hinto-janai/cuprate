use std::sync::Arc;

use cuprate_rpc_interface::RpcError;
use cuprate_rpc_types::json::{
    AddAuxPowRequest, AddAuxPowResponse, BannedRequest, BannedResponse, CalcPowRequest,
    CalcPowResponse, FlushCacheRequest, FlushCacheResponse, FlushTransactionPoolRequest,
    FlushTransactionPoolResponse, GenerateBlocksRequest, GenerateBlocksResponse,
    GetAlternateChainsRequest, GetAlternateChainsResponse, GetBansRequest, GetBansResponse,
    GetBlockCountRequest, GetBlockCountResponse, GetBlockHeaderByHashRequest,
    GetBlockHeaderByHashResponse, GetBlockHeaderByHeightRequest, GetBlockHeaderByHeightResponse,
    GetBlockHeadersRangeRequest, GetBlockHeadersRangeResponse, GetBlockRequest, GetBlockResponse,
    GetCoinbaseTxSumRequest, GetCoinbaseTxSumResponse, GetConnectionsRequest,
    GetConnectionsResponse, GetFeeEstimateRequest, GetFeeEstimateResponse, GetInfoRequest,
    GetInfoResponse, GetLastBlockHeaderRequest, GetLastBlockHeaderResponse, GetMinerDataRequest,
    GetMinerDataResponse, GetOutputHistogramRequest, GetOutputHistogramResponse,
    GetTransactionPoolBacklogRequest, GetTransactionPoolBacklogResponse, GetTxIdsLooseRequest,
    GetTxIdsLooseResponse, GetVersionRequest, GetVersionResponse, HardForkInfoRequest,
    HardForkInfoResponse, JsonRpcRequest, JsonRpcResponse, OnGetBlockHashRequest,
    OnGetBlockHashResponse, PruneBlockchainRequest, PruneBlockchainResponse, RelayTxRequest,
    RelayTxResponse, SetBansRequest, SetBansResponse, SubmitBlockRequest, SubmitBlockResponse,
    SyncInfoRequest, SyncInfoResponse,
};
use tower::ServiceExt;

use crate::rpc::CupratedRpcHandler;

/// Map a [`JsonRpcRequest`] to the function that will lead to a [`JsonRpcResponse`].
pub(super) async fn map_request(
    state: CupratedRpcHandler,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse, RpcError> {
    use JsonRpcRequest as Req;
    use JsonRpcResponse as Resp;

    Ok(match request {
        Req::GetBlockCount(r) => Resp::GetBlockCount(get_block_count(state, r).await?),
        Req::OnGetBlockHash(r) => Resp::OnGetBlockHash(on_get_block_hash(state, r).await?),
        Req::SubmitBlock(r) => Resp::SubmitBlock(submit_block(state, r).await?),
        Req::GenerateBlocks(r) => Resp::GenerateBlocks(generate_blocks(state, r).await?),
        Req::GetLastBlockHeader(r) => {
            Resp::GetLastBlockHeader(get_last_block_header(state, r).await?)
        }
        Req::GetBlockHeaderByHash(r) => {
            Resp::GetBlockHeaderByHash(get_block_header_by_hash(state, r).await?)
        }
        Req::GetBlockHeaderByHeight(r) => {
            Resp::GetBlockHeaderByHeight(get_block_header_by_height(state, r).await?)
        }
        Req::GetBlockHeadersRange(r) => {
            Resp::GetBlockHeadersRange(get_block_headers_range(state, r).await?)
        }
        Req::GetBlock(r) => Resp::GetBlock(get_block(state, r).await?),
        Req::GetConnections(r) => Resp::GetConnections(get_connections(state, r).await?),
        Req::GetInfo(r) => Resp::GetInfo(get_info(state, r).await?),
        Req::HardForkInfo(r) => Resp::HardForkInfo(hard_fork_info(state, r).await?),
        Req::SetBans(r) => Resp::SetBans(set_bans(state, r).await?),
        Req::GetBans(r) => Resp::GetBans(get_bans(state, r).await?),
        Req::Banned(r) => Resp::Banned(banned(state, r).await?),
        Req::FlushTransactionPool(r) => {
            Resp::FlushTransactionPool(flush_transaction_pool(state, r).await?)
        }
        Req::GetOutputHistogram(r) => {
            Resp::GetOutputHistogram(get_output_histogram(state, r).await?)
        }
        Req::GetCoinbaseTxSum(r) => Resp::GetCoinbaseTxSum(get_coinbase_tx_sum(state, r).await?),
        Req::GetVersion(r) => Resp::GetVersion(get_version(state, r).await?),
        Req::GetFeeEstimate(r) => Resp::GetFeeEstimate(get_fee_estimate(state, r).await?),
        Req::GetAlternateChains(r) => {
            Resp::GetAlternateChains(get_alternate_chains(state, r).await?)
        }
        Req::RelayTx(r) => Resp::RelayTx(relay_tx(state, r).await?),
        Req::SyncInfo(r) => Resp::SyncInfo(sync_info(state, r).await?),
        Req::GetTransactionPoolBacklog(r) => {
            Resp::GetTransactionPoolBacklog(get_transaction_pool_backlog(state, r).await?)
        }
        Req::GetMinerData(r) => Resp::GetMinerData(get_miner_data(state, r).await?),
        Req::PruneBlockchain(r) => Resp::PruneBlockchain(prune_blockchain(state, r).await?),
        Req::CalcPow(r) => Resp::CalcPow(calc_pow(state, r).await?),
        Req::FlushCache(r) => Resp::FlushCache(flush_cache(state, r).await?),
        Req::AddAuxPow(r) => Resp::AddAuxPow(add_aux_pow(state, r).await?),
        Req::GetTxIdsLoose(r) => Resp::GetTxIdsLoose(get_tx_ids_loose(state, r).await?),
    })
}

async fn get_block_count(
    state: CupratedRpcHandler,
    request: GetBlockCountRequest,
) -> Result<GetBlockCountResponse, RpcError> {
    todo!()
}

async fn on_get_block_hash(
    state: CupratedRpcHandler,
    request: OnGetBlockHashRequest,
) -> Result<OnGetBlockHashResponse, RpcError> {
    todo!()
}

async fn submit_block(
    state: CupratedRpcHandler,
    request: SubmitBlockRequest,
) -> Result<SubmitBlockResponse, RpcError> {
    todo!()
}

async fn generate_blocks(
    state: CupratedRpcHandler,
    request: GenerateBlocksRequest,
) -> Result<GenerateBlocksResponse, RpcError> {
    todo!()
}

async fn get_last_block_header(
    state: CupratedRpcHandler,
    request: GetLastBlockHeaderRequest,
) -> Result<GetLastBlockHeaderResponse, RpcError> {
    todo!()
}

async fn get_block_header_by_hash(
    state: CupratedRpcHandler,
    request: GetBlockHeaderByHashRequest,
) -> Result<GetBlockHeaderByHashResponse, RpcError> {
    todo!()
}

async fn get_block_header_by_height(
    state: CupratedRpcHandler,
    request: GetBlockHeaderByHeightRequest,
) -> Result<GetBlockHeaderByHeightResponse, RpcError> {
    todo!()
}

async fn get_block_headers_range(
    state: CupratedRpcHandler,
    request: GetBlockHeadersRangeRequest,
) -> Result<GetBlockHeadersRangeResponse, RpcError> {
    todo!()
}

async fn get_block(
    state: CupratedRpcHandler,
    request: GetBlockRequest,
) -> Result<GetBlockResponse, RpcError> {
    todo!()
}

async fn get_connections(
    state: CupratedRpcHandler,
    request: GetConnectionsRequest,
) -> Result<GetConnectionsResponse, RpcError> {
    todo!()
}

async fn get_info(
    state: CupratedRpcHandler,
    request: GetInfoRequest,
) -> Result<GetInfoResponse, RpcError> {
    todo!()
}

async fn hard_fork_info(
    state: CupratedRpcHandler,
    request: HardForkInfoRequest,
) -> Result<HardForkInfoResponse, RpcError> {
    todo!()
}

async fn set_bans(
    state: CupratedRpcHandler,
    request: SetBansRequest,
) -> Result<SetBansResponse, RpcError> {
    todo!()
}

async fn get_bans(
    state: CupratedRpcHandler,
    request: GetBansRequest,
) -> Result<GetBansResponse, RpcError> {
    todo!()
}

async fn banned(
    state: CupratedRpcHandler,
    request: BannedRequest,
) -> Result<BannedResponse, RpcError> {
    todo!()
}

async fn flush_transaction_pool(
    state: CupratedRpcHandler,
    request: FlushTransactionPoolRequest,
) -> Result<FlushTransactionPoolResponse, RpcError> {
    todo!()
}

async fn get_output_histogram(
    state: CupratedRpcHandler,
    request: GetOutputHistogramRequest,
) -> Result<GetOutputHistogramResponse, RpcError> {
    todo!()
}

async fn get_coinbase_tx_sum(
    state: CupratedRpcHandler,
    request: GetCoinbaseTxSumRequest,
) -> Result<GetCoinbaseTxSumResponse, RpcError> {
    todo!()
}

async fn get_version(
    state: CupratedRpcHandler,
    request: GetVersionRequest,
) -> Result<GetVersionResponse, RpcError> {
    todo!()
}

async fn get_fee_estimate(
    state: CupratedRpcHandler,
    request: GetFeeEstimateRequest,
) -> Result<GetFeeEstimateResponse, RpcError> {
    todo!()
}

async fn get_alternate_chains(
    state: CupratedRpcHandler,
    request: GetAlternateChainsRequest,
) -> Result<GetAlternateChainsResponse, RpcError> {
    todo!()
}

async fn relay_tx(
    state: CupratedRpcHandler,
    request: RelayTxRequest,
) -> Result<RelayTxResponse, RpcError> {
    todo!()
}

async fn sync_info(
    state: CupratedRpcHandler,
    request: SyncInfoRequest,
) -> Result<SyncInfoResponse, RpcError> {
    todo!()
}

async fn get_transaction_pool_backlog(
    state: CupratedRpcHandler,
    request: GetTransactionPoolBacklogRequest,
) -> Result<GetTransactionPoolBacklogResponse, RpcError> {
    todo!()
}

async fn get_miner_data(
    state: CupratedRpcHandler,
    request: GetMinerDataRequest,
) -> Result<GetMinerDataResponse, RpcError> {
    todo!()
}

async fn prune_blockchain(
    state: CupratedRpcHandler,
    request: PruneBlockchainRequest,
) -> Result<PruneBlockchainResponse, RpcError> {
    todo!()
}

async fn calc_pow(
    state: CupratedRpcHandler,
    request: CalcPowRequest,
) -> Result<CalcPowResponse, RpcError> {
    todo!()
}

async fn flush_cache(
    state: CupratedRpcHandler,
    request: FlushCacheRequest,
) -> Result<FlushCacheResponse, RpcError> {
    todo!()
}

async fn add_aux_pow(
    state: CupratedRpcHandler,
    request: AddAuxPowRequest,
) -> Result<AddAuxPowResponse, RpcError> {
    todo!()
}

async fn get_tx_ids_loose(
    state: CupratedRpcHandler,
    request: GetTxIdsLooseRequest,
) -> Result<GetTxIdsLooseResponse, RpcError> {
    todo!()
}