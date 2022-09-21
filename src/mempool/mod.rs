use crate::{
    Batch, Config, ConsensusMempoolMsg, Helper, MempoolHandler, MempoolMsg, Processor,
    Synchronizer, Transaction, TxReceiveHandler,
};
use libcrypto::hash::Hash;
use network::{
    plaintcp::{TcpReceiver, TcpSimpleSender},
    Acknowledgement, Identifier,
};
use std::net::SocketAddr;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub struct Mempool<Id, Round, Storage, Tx> {
    /// The Id of this server
    my_name: Id,
    /// The Ids of all the servers
    all_ids: Vec<Id>,
    /// The parameters for the mempool
    params: Config<Round>,
    /// The DB implementation to handle new transactions
    store: Storage,
    /// The networking object to send mempool messages to other mempools
    mempool_sender: TcpSimpleSender<Id, MempoolMsg<Id, Tx>, Acknowledgement>,
    /// Address where this mempool should listen to requests from other mempools
    mempool_addr: SocketAddr,
    /// Address where this mempool should listen to requests from clients
    client_addr: SocketAddr,
}

impl<Id, Round, Storage, Tx> Mempool<Id, Round, Storage, Tx>
where
    Id: Identifier,
    Round: crate::Round,
    Storage: libstorage::Store,
    Tx: Transaction,
{
    pub fn spawn(
        my_name: Id,
        all_ids: Vec<Id>,
        params: Config<Round>,
        store: Storage,
        mempool_sender: TcpSimpleSender<Id, MempoolMsg<Id, Tx>, Acknowledgement>,
        rx_consensus: UnboundedReceiver<ConsensusMempoolMsg<Id, Round>>,
        // Output to batcher that we have a client tx
        tx_batcher: UnboundedSender<(Tx, /* Size of the tx */ usize)>,
        // The consensus creates this so it can forward it to the batcher
        tx_processor: UnboundedSender<Batch<Tx>>,
        // The consensus will let us know once a batch is ready to be proposed
        rx_processor: UnboundedReceiver<Batch<Tx>>,
        // The consensus will respond to processed batches
        tx_consensus: UnboundedSender<Hash>,
        mempool_addr: SocketAddr,
        client_addr: SocketAddr,
    ) {
        // NOTE: This log entry is used to compute performance.
        params.log();

        let mut mempool = Self {
            my_name,
            all_ids,
            params,
            store,
            mempool_sender,
            mempool_addr,
            client_addr,
        };

        mempool.handle_client_messages(
            tx_batcher,   // Output client tx [to batcher]
            rx_processor, // Input ready batches [from batcher] to the processor
            tx_consensus, // Output batch hash [to consensus]
        );

        mempool.handle_mempool_messages(tx_processor);

        mempool.handle_consensus_messages(rx_consensus);
    }

    /// Spawn all tasks responsible to handle messages from the consensus.
    fn handle_consensus_messages(
        self,
        rx_consensus: UnboundedReceiver<ConsensusMempoolMsg<Id, Round>>,
    ) {
        Synchronizer::spawn(
            self.my_name,
            rx_consensus,
            self.params.gc_depth,
            self.mempool_sender,
            self.store.clone(),
            self.params.sync_retry_delay,
            self.all_ids.clone(),
            self.params.sync_retry_nodes,
        );
    }

    /// Spawn all tasks responsible to handle clients transactions.
    fn handle_client_messages(
        &self,
        // Output client transactions to batcher
        tx_batcher: UnboundedSender<(Tx, usize)>,
        // Receive batches and process them
        rx_processor: UnboundedReceiver<Batch<Tx>>,
        tx_consensus: UnboundedSender<Hash>,
    ) {
        // Handle transactions sent by the client
        TcpReceiver::spawn(self.client_addr, TxReceiveHandler::new(tx_batcher));

        // The above will be forwarded to the batcher
        // Once the batcher has a batch ready, it will send it to the processor

        // Start the processor
        Processor::spawn(
            self.store.clone(),
            rx_processor, // From the batcher
            tx_consensus, // Output to
        );
    }

    fn handle_mempool_messages(
        &mut self,
        tx_processor: UnboundedSender<Batch<Tx>>,
    ) {
        let (tx_helper, rx_helper) = unbounded_channel();

        Helper::<Id, Storage, Tx>::spawn(
            TcpSimpleSender::with_peers(self.mempool_sender.get_peers()),
            rx_helper,
            self.store.clone(),
        );

        TcpReceiver::spawn(
            self.mempool_addr,
            MempoolHandler::new(tx_helper, tx_processor),
        );
    }
}
