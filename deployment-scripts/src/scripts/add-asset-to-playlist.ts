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
    let publishId = await channelHelper.PublishAsset(context, "creator", channelId, collectionId, assetId, true);

    // Add the asset to the playlist
    await channelHelper.AddAssetToPlaylist(context, "creator", channelId, channelId, playlistName, publishId);

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, playlistName);

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, playlistName);
}

AddAssetToPlaylist();