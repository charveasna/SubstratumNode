// Copyright (c) 2017-2019, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.

use crate::sub_lib::blockchain_bridge::BlockchainBridgeConfig;
use crate::sub_lib::blockchain_bridge::BlockchainBridgeSubs;
use crate::sub_lib::blockchain_bridge::ReportAccountsPayable;
use crate::sub_lib::cryptde::CryptDE;
use crate::sub_lib::cryptde::PlainData;
use crate::sub_lib::cryptde_null::CryptDENull;
use crate::sub_lib::logger::Logger;
use crate::sub_lib::peer_actors::BindMessage;
use actix::Actor;
use actix::Addr;
use actix::Context;
use actix::Handler;

pub struct BlockchainBridge {
    config: BlockchainBridgeConfig,
    logger: Logger,
}

impl Actor for BlockchainBridge {
    type Context = Context<Self>;
}

impl Handler<BindMessage> for BlockchainBridge {
    type Result = ();

    fn handle(&mut self, _msg: BindMessage, _ctx: &mut Self::Context) -> Self::Result {
        match self.config.consuming_private_key.as_ref() {
            Some(key) => {
                // This is hashing the UTF-8 bytes of the string, not the actual bytes encoded as hex
                let hash = CryptDENull::new().hash(&PlainData::new(key.as_bytes()));
                self.logger.debug(format!(
                    "Received BindMessage; consuming private key that hashes to {:?}",
                    hash
                ));
            }
            None => {
                self.logger
                    .debug("Received BindMessage; no consuming private key specified".to_string());
            }
        }
    }
}

impl Handler<ReportAccountsPayable> for BlockchainBridge {
    type Result = ();

    fn handle(&mut self, _msg: ReportAccountsPayable, _ctx: &mut Self::Context) -> Self::Result {
        self.logger
            .debug("Received ReportAccountsPayable message".to_string());
    }
}

impl BlockchainBridge {
    pub fn new(config: BlockchainBridgeConfig) -> BlockchainBridge {
        BlockchainBridge {
            config,
            logger: Logger::new("BlockchainBridge"),
        }
    }

    pub fn make_subs_from(addr: &Addr<BlockchainBridge>) -> BlockchainBridgeSubs {
        BlockchainBridgeSubs {
            bind: addr.clone().recipient::<BindMessage>(),
            report_accounts_payable: addr.clone().recipient::<ReportAccountsPayable>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::logging::init_test_logging;
    use crate::test_utils::logging::TestLogHandler;
    use crate::test_utils::recorder::peer_actors_builder;
    use crate::test_utils::test_utils::cryptde;
    use actix::Addr;
    use actix::System;

    #[test]
    fn blockchain_bridge_receives_bind_message_with_consuming_private_key() {
        init_test_logging();

        let consuming_private_key =
            "cc46befe8d169b89db447bd725fc2368b12542113555302598430cb5d5c74ea9".to_string();
        let subject = BlockchainBridge::new(BlockchainBridgeConfig {
            consuming_private_key: Some(consuming_private_key.clone()),
        });

        let system = System::new("blockchain_bridge_receives_bind_message");
        let addr: Addr<BlockchainBridge> = subject.start();

        addr.try_send(BindMessage {
            peer_actors: peer_actors_builder().build(),
        })
        .unwrap();

        System::current().stop();
        system.run();
        let hash = cryptde().hash(&PlainData::new(consuming_private_key.as_bytes()));
        TestLogHandler::new()
            .exists_log_containing(&format!("DEBUG: BlockchainBridge: Received BindMessage; consuming private key that hashes to {:?}", hash));
    }

    #[test]
    fn blockchain_bridge_receives_bind_message_without_consuming_private_key() {
        init_test_logging();

        let subject = BlockchainBridge::new(BlockchainBridgeConfig {
            consuming_private_key: None,
        });

        let system = System::new("blockchain_bridge_receives_bind_message");
        let addr: Addr<BlockchainBridge> = subject.start();

        addr.try_send(BindMessage {
            peer_actors: peer_actors_builder().build(),
        })
        .unwrap();

        System::current().stop();
        system.run();
        TestLogHandler::new().exists_log_containing(
            "DEBUG: BlockchainBridge: Received BindMessage; no consuming private key specified",
        );
    }

    #[test]
    fn blockchain_bridge_receives_report_accounts_payable_message_and_logs() {
        init_test_logging();

        let subject = BlockchainBridge::new(BlockchainBridgeConfig {
            consuming_private_key: None,
        });

        let system = System::new("blockchain_bridge_receives_report_accounts_payable_message");
        let addr: Addr<BlockchainBridge> = subject.start();

        addr.try_send(ReportAccountsPayable { accounts: vec![] })
            .unwrap();

        System::current().stop_with_code(0);
        system.run();
        TestLogHandler::new().exists_log_containing(
            "DEBUG: BlockchainBridge: Received ReportAccountsPayable message",
        );
    }
}
