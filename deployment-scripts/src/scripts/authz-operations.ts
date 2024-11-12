import Context, { CONTRACT_MAP } from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';
import AuthzHelper from '../helpers/authz.helper';
import OnftHelper from '../helpers/onft.helper';


const authzOperationPublish = async () => {
    let context = new Context;
    await context.initialize();
    let authzHelper = new AuthzHelper;
    let onftHelper = new OnftHelper;
    let channelHelper = new ChannelHelper;


    await channelHelper.InstantiateChannelContract(context);
    let testing_user_name = "creator" + Math.floor(Math.random() * 1000);

    // Create new channel
    let channel_id = await channelHelper.CreateChannel(context, "creator", testing_user_name,)

    // Create an ONFT collection
    logger.log(1, `Creating ONFT collection with id: ${channel_id}`);
    logger.log(1, `This will serve as the collection for the asset being published`);
    let testingCollectionId = "OmniflixTestingCollection" + Math.floor(Math.random() * 10000);
    let collectionId = await onftHelper.createOnftCollection(context, "creator", testingCollectionId);

    // Mint an Asset from the ONFT collection
    // Creator mints an asset.
    logger.log(1, `Minting asset for collection with id: ${collectionId}`);
    let assetId = "OmniflixTestingAsset" + Math.floor(Math.random() * 10000);
    await onftHelper.mintOnft(context, "creator", "creator", collectionId, assetId);

    logger.log(1, `Asset and channel is owned by creator`);
    let publishMsg = {
        publish: {
            asset_onft_collection_id: collectionId,
            asset_onft_id: assetId,
            channel_id: channel_id,
            is_visible: true,
            salt: context.generateRandomSalt(5)
        }
    }
    // Test auth from admin to creator
    logger.log(1, `Granting authorization from creator to viewer`);
    await authzHelper.giveAuthorization(context, 'creator', 'viewer');
    logger.log(1, `Viewer now can execute on behalf of creator`);
    await authzHelper.executeMsgWithAuthz(context, 'creator', "viewer", context.getContractAddress(CONTRACT_MAP.OMNIFLIX_CHANNEL), '10', publishMsg);
}


authzOperationPublish().then(() => {
    logger.info('Channel created successfully');
}).catch((error) => {
    logger.error(`Error: ${error}`);
});

