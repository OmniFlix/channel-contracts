import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';
import AuthzHelper from '../helpers/authz.helper';


const authzOperations = async () => {
    let context = new Context;
    await context.initialize();
    let authzHelper = new AuthzHelper;

    // Test auth from admin to creator
    await authzHelper.giveAuthorization(context, 'admin', 'creator');

}


authzOperations().then(() => {
    logger.info('Channel created successfully');
}).catch((error) => {
    logger.error(`Error creating channel: ${error}`);
});

