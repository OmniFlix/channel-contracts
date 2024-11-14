import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import OnftHelper from '../helpers/onft.helper';
import { logger } from '../utils/logger';

const manageAssetsInPlaylist = async () => {
    let context = new Context;
    await context.initialize();

    let channelHelper = new ChannelHelper;
    let onftHelper = new OnftHelper;
    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    let testingUserName = "creator" + Math.floor(Math.random() * 1000);

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", testingUserName);

    let playlistName = "PlaylistTest";

    let collectionId = "CollectionTest" + Math.floor(Math.random() * 1000);
    let assetId = "AssetTest" + Math.floor(Math.random() * 1000);

    // Create an ONFT collection
    await onftHelper.createOnftCollection(context, "creator", collectionId);

    // Create a asset
    await onftHelper.mintOnft(context, "creator", "creator", collectionId, assetId);

    // Create another asset
    let assetId2 = "AssetTest" + Math.floor(Math.random() * 1000);
    await onftHelper.mintOnft(context, "creator", "creator", collectionId, assetId2);

    // Creating a playlist
    await channelHelper.CreatePlaylist(context, "creator", channelId, playlistName);

    // Publish the asset
    let publishId = await channelHelper.PublishAsset(context, "creator", channelId, collectionId, assetId, true);

    // Publish the asset
    let publishId2 = await channelHelper.PublishAsset(context, "creator", channelId, collectionId, assetId2, true);

    // Add the asset to the playlist
    await channelHelper.AddAssetToPlaylist(context, "creator", channelId, channelId, playlistName, publishId);

    // Add the asset to the playlist
    await channelHelper.AddAssetToPlaylist(context, "creator", channelId, channelId, playlistName, publishId2);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);

    // Update the asset in the playlist
    await channelHelper.UpdateAsset(context, "creator", channelId, publishId, false);

    // Update the asset in the playlist
    await channelHelper.UpdateAsset(context, "creator", channelId, publishId2, false);

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, playlistName);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);

    // Update the asset in the playlist
    await channelHelper.UpdateAsset(context, "creator", channelId, publishId, true);

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, playlistName);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);

    // Add the asset to the playlist
    await channelHelper.AddAssetToPlaylist(context, "creator", channelId, channelId, playlistName, publishId);

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, playlistName);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);

    // Unpublish the asset
    await channelHelper.UnpublishAsset(context, "creator", channelId, publishId);

    // Query Asset
    try {
        await channelHelper.QueryAsset(context, "creator", channelId, publishId);
    } catch (error) {
        logger.log(1, `Expected Error: ${error}`);
    }

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, playlistName);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);

}

manageAssetsInPlaylist();
