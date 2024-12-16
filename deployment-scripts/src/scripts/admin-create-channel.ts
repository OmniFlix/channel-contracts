import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';


const adminCreateChannel = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;
    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);

    // Default reserved username is 'reserved'
    // User can not use this username
    await channel_helper.CreateChannel(context, "creator", "reserved").catch((error) => {
        logger.log(1, 'Error creating reserved channel as expected');
    });

    // admin can create reserved channel
    await channel_helper.AdminCreateChannel(context, "reserved", "creator");
}


adminCreateChannel().then(() => {
    logger.info('Admin create channel success');
}).catch((error) => {
    logger.error('Admin create channel failed', error);
});

