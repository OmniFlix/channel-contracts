import { InstantiateOptions, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { deploymentConfig } from '../../configs/deployment_config.ts'
import chainConfig from '../../configs/chain_config.json'
import Context from './context.ts'
import _, { random } from 'lodash'
import assert from 'assert'
import { CONTRACT_MAP } from './context.ts'
import { logger } from '../utils/logger.ts'
import { InstantiateMsg, Coin } from '../types/OmniFlixChannel.types.ts'
import { OmniFlixChannelClient } from '../types/OmniFlixChannel.client.ts'


export default class ChannelHelper {
    InstantiateChannelContract = async (context: Context) => {
        let { client, address: sender } = context.getTestUser('admin')
        let fee_collector_address = context.getTestUser('fee_collector').address
        let instantiateOptions: InstantiateOptions = {
            funds: [
                {
                    amount: deploymentConfig.onft_collection_creator_fee,
                    denom: chainConfig.denom,
                },
            ],
        }
        let instantiateMsg: InstantiateMsg = {
            fee_collector: fee_collector_address,
            admin: sender,
            channel_creation_fee: [{
                amount: deploymentConfig.channel_creation_fee,
                denom: chainConfig.denom,
            }],
            channels_collection_id: deploymentConfig.channels_collection_id + random(10000000).toString(),
            channels_collection_name: deploymentConfig.channels_collection_name,
            channels_collection_symbol: deploymentConfig.channels_collection_symbol,
            reserved_usernames: ["reserved"]
        }

        let instantiateResult = await client.instantiate(
            sender,
            context.getCodeId(CONTRACT_MAP.OMNIFLIX_CHANNEL),
            instantiateMsg,
            'OmniFlix Channel',
            'auto',
            instantiateOptions,
        )

        _.forEach(instantiateResult.events, (event) => {
            if (event.type === 'instantiate') {
                let codeId = parseInt(event.attributes[1].value, 10)
                let contractKey = context.getContractKeyByCodeId(codeId)
                assert(contractKey, 'contract address not found in wasm event attributes')
                logger.log(1, `Channel contract address: ${event.attributes[0].value}`)
                logger.log(1, `Tx_Hash: ${instantiateResult.transactionHash}\n`)
                context.addContractAddress(contractKey, event.attributes[0].value)
            }
        })
        return instantiateResult;
    }

    SetConfig = async (context: Context, account_name: string, admin?: string, channel_creation_fee?: string, fee_collector?: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));

        let executeMsg: {
            set_config: {
                admin?: string;
                channelCreationFee?: Coin[];
                fee_collector?: string;
            }
        } = {
            set_config: {}
        };

        if (admin) {
            executeMsg.set_config.admin = admin
        }

        if (channel_creation_fee) {
            executeMsg.set_config.channelCreationFee = [{
                amount: channel_creation_fee,
                denom: chainConfig.denom,
            }]
        }

        if (fee_collector) {
            let fee_collector_address = context.getTestUser(fee_collector).address
            executeMsg.set_config.fee_collector = fee_collector_address
        }

        let res = await channel_client.setConfig(executeMsg.set_config)
        logger.log(1, `Channel config updated`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return res;
    }

    CreateChannel = async (context: Context, account_name: string, user_name: string, collaborators?: []) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelCreate({
            userName: user_name,
            description: "OmniFlix Channel Testing",
            salt: context.generateRandomSalt(5),
            collaborators: collaborators,
        }, "auto", "", [
            {
                amount: deploymentConfig.channel_creation_fee,
                denom: chainConfig.denom,
            },
        ]);
        let channel_id = context.getEventAttribute(res, undefined, 'channel_id');
        logger.log(1, `Channel created with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return channel_id;
    }
    AdminCreateChannel = async (context: Context, user_name: string, recipient: string, collaborators?: []) => {
        let { client, address: senderAddress } = context.getTestUser('admin');
        let { client: recipientClient, address: recipientAddress } = context.getTestUser(recipient);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.adminChannelCreate({
            userName: user_name,
            recipient: recipientAddress,
            description: "OmniFlix Channel Testing",
            salt: context.generateRandomSalt(5),
            collaborators: collaborators,
        }
        );
        let channel_id = context.getEventAttribute(res, undefined, 'channel_id');
        logger.log(1, `Channel created with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return channel_id;
    }


    PublishAsset = async (context: Context, account_name: string, channel_id: string, asset_onft_collection_id: string, asset_onft_id: string, is_visible: boolean, playlist_name?: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.publish({
            assetType: {
                nft: {
                    collection_id: asset_onft_collection_id,
                    onft_id: asset_onft_id,
                }
            },
            channelId: channel_id,
            isVisible: is_visible,
            salt: context.generateRandomSalt(5),
            playlistName: playlist_name,
        });
        let publishId = context.getEventAttribute(res, undefined, 'publish_id');
        logger.log(1, `Asset published with id: ${publishId}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return publishId;
    }
    PublishOffchainAsset = async (context: Context, account_name: string, channel_id: string, asset_url: string, asset_name: string, asset_description: string, is_visible: boolean, playlist_name?: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.publish({
            assetType: {
                off_chain: {
                    media_uri: asset_url,
                    name: asset_name,
                    description: asset_description,
                }
            },
            channelId: channel_id,
            isVisible: is_visible,
            salt: context.generateRandomSalt(5),
            playlistName: playlist_name,
        });
        let publishId = context.getEventAttribute(res, undefined, 'publish_id');
        logger.log(1, `Asset published with id: ${publishId}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return publishId;
    }

    CreatePlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistCreate({
            channelId: channel_id,
            playlistName: playlist_name,
        });
        logger.log(1, `Playlist created with name ${playlist_name} under channel id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    AddAssetToPlaylist = async (context: Context, account_name: string, channel_id: string, asset_channel_id: string, playlist_name: string, publish_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistAddAsset({
            assetChannelId: asset_channel_id,
            channelId: channel_id,
            playlistName: playlist_name,
            publishId: publish_id,
        });
        logger.log(1, `Asset added to playlist: ${playlist_name} with id: ${publish_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    RefreshPlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistRefresh({
            channelId: channel_id,
            playlistName: playlist_name,
        });
        logger.log(1, `Playlist refreshed with name ${playlist_name} under channel id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    DeletePlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistDelete({
            channelId: channel_id,
            playlistName: playlist_name,
        });
        logger.log(1, `Playlist deleted with name ${playlist_name} under channel id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    RemoveAssetFromPlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string, publish_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistRemoveAsset({
            channelId: channel_id,
            playlistName: playlist_name,
            publishId: publish_id,
        });
        logger.log(1, `Asset removed from playlist: ${playlist_name} with id: ${publish_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    UnpublishAsset = async (context: Context, account_name: string, channel_id: string, publish_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.unpublish({
            channelId: channel_id,
            publishId: publish_id,
        });
        logger.log(1, `Asset unpublished with id: ${publish_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    UpdateChannelDetails = async (context: Context, account_name: string, channel_id: string, description: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelUpdateDetails({
            channelId: channel_id,
            description: description,
        });
        logger.log(1, `Channel details updated with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    DeleteChannel = async (context: Context, account_name: string, channel_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelDelete({
            channelId: channel_id,
        });
        logger.log(1, `Channel deleted with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }

    GetChannelDetails = async (context: Context, account_name: string, channel_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelDetails({
            channelId: channel_id,
        });
        logger.log(1, `Channel details: ${JSON.stringify(res)}`)
        return res;
    }
    UpdateAsset = async (context: Context, account_name: string, channel_id: string, publish_id: string, is_visible: boolean) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.assetUpdateDetails({
            channelId: channel_id,
            isVisible: is_visible,
            publishId: publish_id,
        });
        logger.log(1, `Asset updated with id: ${publish_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
    }
    QueryAssets = async (context: Context, channel_id: string) => {
        let { client, address: senderAddress } = context.getTestUser("admin");
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.assets({
            channelId: channel_id,
        });
        logger.log(1, `Assets: ${JSON.stringify(res)}`)
        return res;
    }
    QueryAsset = async (context: Context, account_name: string, channel_id: string, publish_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.asset({
            channelId: channel_id,
            publishId: publish_id,
        });
        logger.log(1, `Asset: ${JSON.stringify(res)}`)
        return res;
    }
    QueryPlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlist({
            channelId: channel_id,
            playlistName: playlist_name,
        });
        logger.log(1, `Playlists for channel_id: ${channel_id}: ${JSON.stringify(res)}`)
        return res;
    }
    QueryChannelConfig = async (context: Context, account_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.config();
        logger.log(1, `Channel config: ${JSON.stringify(res)}`)
        return res;
    }
}