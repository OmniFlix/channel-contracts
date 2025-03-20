import { InstantiateOptions, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { deploymentConfig } from '../../configs/deployment_config.ts'
import chainConfig from '../../configs/chain_config.json'
import Context from './context.ts'
import _, { random } from 'lodash'
import assert from 'assert'
import { CONTRACT_MAP } from './context.ts'
import { logger } from '../utils/logger.ts'
import { InstantiateMsg, Coin, ReservedUsername, AssetSource, Flag } from '../types/OmniFlixChannel.types.ts'
import { OmniFlixChannelClient } from '../types/OmniFlixChannel.client.ts'


export default class ChannelHelper {
    InstantiateChannelContract = async (context: Context) => {
        let { client, address: sender } = context.getTestUser('admin')
        let fee_collector_address = context.getTestUser('fee_collector').address
        let instantiateOptions: InstantiateOptions = {
            funds: [
                {
                    amount: deploymentConfig.onft_collection_creation_fee,
                    denom: chainConfig.denom,
                },
            ],
        }
        let instantiateMsg: InstantiateMsg = {
            fee_collector: fee_collector_address,
            protocol_admin: sender,
            channel_creation_fee: [{
                amount: deploymentConfig.channel_creation_fee,
                denom: chainConfig.denom,
            }],
            channels_collection_details: {
                collection_id: deploymentConfig.channels_collection_id + random(10000000).toString(),
                collection_name: deploymentConfig.channels_collection_name,
                collection_symbol: deploymentConfig.channels_collection_symbol,
                data: "test".toString(),
                description: "test".toString(),
                preview_uri: "https://www.omniflix.network",
                schema: "test".toString(),
                uri: "https://www.omniflix.network",
                uri_hash: "test".toString(),
            },
            channel_token_details: {
                media_uri: "https://www.omniflix.network",
                preview_uri: "https://www.omniflix.network",
                description: "test".toString(),
                extensible: false,
                nsfw: false,
                royalty_share: "0".toString(),
                transferable: true,
                uri_hash: "test".toString(),
            },
            reserved_usernames: [
                // Set admin as reserved username but set address as empty string
                {
                    username: "admin",
                },
                // Set fee_collector as reserved username
                {
                    username: "feecollector",
                    address: fee_collector_address,
                },
                {
                    username: "reserved",
                },
            ],
            accepted_tip_denoms: [
                chainConfig.denom,
            ],
        }

        let instantiateResult = await client.instantiate(
            sender,
            context.getCodeId(CONTRACT_MAP.OMNIFLIX_CHANNEL),
            instantiateMsg,
            'OmniFlix Channel',
            'auto',
            instantiateOptions,
        )
        logger.log(1, `Gas used: ${instantiateResult.gasUsed}`)

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

        let res = await channel_client.adminSetConfig(executeMsg.set_config)
        logger.log(1, `Channel config updated`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
        return res;
    }

    CreateChannel = async (context: Context, account_name: string, user_name: string, collaborators?: []) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelCreate({
            userName: user_name,
            description: "OmniFlix Channel Testing",
            salt: context.generateRandomSalt(5),
            channelName: user_name,
            bannerPicture: "https://www.omniflix.network",
            profilePicture: "https://www.omniflix.network",
            paymentAddress: senderAddress,
        }, "auto", "", [
            {
                amount: deploymentConfig.channel_creation_fee,
                denom: chainConfig.denom,
            },
        ]);
        let channel_id = context.getEventAttribute(res, undefined, 'channel_id');
        logger.log(1, `Channel created with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
        return channel_id;
    }


    PublishAsset = async (context: Context, account_name: string, channel_id: string, asset_source: AssetSource, is_visible: boolean, playlist_name?: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.assetPublish({
            assetSource: asset_source,
            channelId: channel_id,
            isVisible: is_visible,
            salt: context.generateRandomSalt(5),
            playlistName: playlist_name,
            description: "test".toString(),
            mediaUri: "https://www.omniflix.network",
            name: "test".toString(),
        });
        let publishId = context.getEventAttribute(res, undefined, 'publish_id');
        logger.log(1, `Asset published with id: ${publishId}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
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
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
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
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    RefreshPlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistRefresh({
            channelId: channel_id,
            playlistName: playlist_name,
        });
        logger.log(1, `Playlist refreshed with name ${playlist_name} under channel id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    DeletePlaylist = async (context: Context, account_name: string, channel_id: string, playlist_name: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.playlistDelete({
            channelId: channel_id,
            playlistName: playlist_name,
        });
        logger.log(1, `Playlist deleted with name ${playlist_name} under channel id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
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
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    UnpublishAsset = async (context: Context, account_name: string, channel_id: string, publish_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.assetUnpublish({
            channelId: channel_id,
            publishId: publish_id,
        });
        logger.log(1, `Asset unpublished with id: ${publish_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    UpdateChannelDetails = async (context: Context, account_name: string, channel_id: string, description?: string, channel_name?: string, banner_picture?: string, profile_picture?: string, collaborators?: string[]) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelUpdateDetails({
            channelId: channel_id,
            description: description,
            channelName: channel_name,
            bannerPicture: banner_picture,
            profilePicture: profile_picture,
        });
        logger.log(1, `Channel details updated with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    DeleteChannel = async (context: Context, account_name: string, channel_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelDelete({
            channelId: channel_id,
        });
        logger.log(1, `Channel deleted with id: ${channel_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }
    // Reserved usernames type: [string, string][]
    AddReservedUsernames = async (context: Context, reserved_usernames: ReservedUsername[]) => {
        let { client, address: senderAddress } = context.getTestUser('admin');
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));

        let res = await channel_client.adminManageReservedUsernames({
            addUsernames: reserved_usernames,
        });
        logger.log(1, `Reserved usernames added: ${JSON.stringify(reserved_usernames)}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    AdminRemoveAssets = async (context: Context, account_name: string, asset_keys: string[][], refresh_flags?: boolean) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.adminRemoveAssets({
            assetKeys: asset_keys,
            refreshFlags: refresh_flags
        });
        logger.log(1, `Assets removed: ${JSON.stringify(asset_keys)}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    FlagAsset = async (context: Context, account_name: string, channel_id: string, publish_id: string, flag: Flag) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.assetFlag({
            channelId: channel_id,
            publishId: publish_id,
            flag: flag,
        });
        logger.log(1, `Asset flagged with id: ${publish_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
    }

    QueryChannelDetails = async (context: Context, account_name: string, channel_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelDetails({
            channelId: channel_id,
        });
        logger.log(1, `Channel details: ${JSON.stringify(res)}`)
        return res;
    }
    QueryChannelMetadata = async (context: Context, account_name: string, channel_id: string) => {
        let { client, address: senderAddress } = context.getTestUser(account_name);
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.channelMetadata({
            channelId: channel_id,
        });
        logger.log(1, `Channel metadata: ${JSON.stringify(res)}`)
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
        logger.log(1, `Tx_Hash: ${res.transactionHash}`)
        logger.log(1, `Gas used: ${res.gasUsed}`)
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
    QueryReservedUsernames = async (context: Context, start_after?: string, limit?: number) => {
        let { client, address: senderAddress } = context.getTestUser("admin");
        let channel_client: OmniFlixChannelClient = new OmniFlixChannelClient(client, senderAddress, context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL));
        let res = await channel_client.reservedUsernames({
            startAfter: start_after,
            limit: limit,
        });
        logger.log(1, `Reserved usernames: ${JSON.stringify(res)}`)
        return res;
    }

    async FollowChannel(context: Context, follower: string, channelId: string) {
        const client = await context.getTestUser(follower).client;
        const channelContract = context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL);

        const msg = {
            channel_follow: {
                channel_id: channelId
            }
        };

        const result = await client.execute(
            context.getTestUser(follower).address,
            channelContract,
            msg,
            'auto'
        );

        return result;
    }

    async QueryFollowersCount(context: Context, channelId: string): Promise<number> {
        const client = await context.getTestUser("admin").client;
        const channelContract = context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL);

        const msg = {
            followers_count: {
                channel_id: channelId
            }
        };

        const result = await client.queryContractSmart(channelContract, msg);
        return result;
    }

    async QueryFollowers(context: Context, channelId: string, startAfter?: string, limit?: number): Promise<string[]> {
        const client = await context.getTestUser("admin").client;
        const channelContract = context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL);

        const msg = {
            followers: {
                channel_id: channelId,
                start_after: startAfter,
                limit: limit
            }
        };

        const result = await client.queryContractSmart(channelContract, msg);
        return result;
    }

}