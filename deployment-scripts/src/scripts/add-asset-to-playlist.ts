import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import OnftHelper from '../helpers/onft.helper';
import { logger } from '../utils/logger';

const AddAssetToPlaylist = async () => {
    let context = new Context;
    await context.initialize();

    let channelHelper = new ChannelHelper;
    let onftHelper = new OnftHelper;
    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    let testing_user_name = "creator" + Math.floor(Math.random() * 1000);

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", testing_user_name);

    let playlistName = "PlaylistTest";

    let collectionId = "CollectionTest" + Math.floor(Math.random() * 1000);
    let assetId = "AssetTest" + Math.floor(Math.random() * 1000);

    // Create an ONFT collection
    await onftHelper.createOnftCollection(context, "creator", collectionId);

    // Create a asset
    await onftHelper.mintOnft(context, "creator", "creator", collectionId, assetId);

    // Creating a playlist
    await channelHelper.CreatePlaylist(context, "creator", channelId, playlistName);

    // Publish the asset
    let publishId = await channelHelper.PublishAsset(context, "creator", channelId, {
        nft: {
            collection_id: collectionId,
            onft_id: assetId,
        }
    }, true);

    // Add the asset to the playlist
    await channelHelper.AddAssetToPlaylist(context, "creator", channelId, channelId, playlistName, publishId);

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, playlistName);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);
}

const addAssetFromDiffirentChannel = async () => {
    let context = new Context;
    await context.initialize();

    let channelHelper = new ChannelHelper;
    let onftHelper = new OnftHelper;

    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    // Create two channels one will be the owner of the asset and the other will be the owner of the playlist
    let channelOwnerUserName = "creator" + Math.floor(Math.random() * 1000);
    let playlistOwnerUserName = "creator" + Math.floor(Math.random() * 1000);

    // Create new channel
    let channelOwnerId = await channelHelper.CreateChannel(context, "creator", channelOwnerUserName);
    let playlistOwnerId = await channelHelper.CreateChannel(context, "admin", playlistOwnerUserName);

    let playlistName = "PlaylistTest";

    let collectionId = "CollectionTest" + Math.floor(Math.random() * 1000);
    let assetId = "AssetTest" + Math.floor(Math.random() * 1000);

    // Create an ONFT collection
    await onftHelper.createOnftCollection(context, "creator", collectionId);

    // Create a asset
    await onftHelper.mintOnft(context, "creator", "creator", collectionId, assetId);

    // Creating a playlist
    await channelHelper.CreatePlaylist(context, "admin", playlistOwnerId, playlistName);

    // Publish the asset
    let publishId = await channelHelper.PublishAsset(context, "creator", channelOwnerId, {
        nft: {
            collection_id: collectionId,
            onft_id: assetId,
        }
    }, true);

    // Add the asset to the playlist
    await channelHelper.AddAssetToPlaylist(context, "admin", playlistOwnerId, channelOwnerId, playlistName, publishId);
}

//AddAssetToPlaylist();
addAssetFromDiffirentChannel();