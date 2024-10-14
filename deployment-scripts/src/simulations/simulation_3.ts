import Controller from '../helpers/channel.helper';
import TokenFactory from '../helpers/onft.helper';
import Stream from '../helpers/stream.helper';
import { logger } from '../utils/logger';
import { deploymentConfig } from '../../configs/deployment_config';
import { sleepUntil } from '../utils/time';
import Context from '../helpers/context';
import { getFutureTimestamp, nanoToDate } from '../utils/datetime'


const simulation_3 = async () => {
    let context = new Context;
    let controller = new Controller;
    let stream = new Stream;
    let tokenFactory = new TokenFactory;

    await context.initialize();
    // Simulation 3: 
    // Create a stream with threshold set



    // Details:
    // 1. Create a stream with threshold set
    // 1.1. Threshold amount is 1000000000000000
    // 1.2. Stream waiting duration is 10 seconds.
    // 1.3. Stream bootstraping duration is 20 seconds.
    // 1.4. Stream duration is 30 seconds.

    // 2. Subscribe to the stream at bootstraping. <Subsriber1> with 5_000_000 amount.
    // 3. Synch subsriber1 position.
    // 4. Subscibe to the stream at start time. <Subsriber2> with 5_000_000 amount.
    // 5. Synch subsriber2 position.
    // 6. Stream ends but threshold is not reached.
    // 7. Subsriber1 exits the stream after it ends.
    // 8. Creator cancels the stream after it ends.
    // 9. Subsriber2 exits the stream after it ends.

    await controller.InstantiateStreamSwapController(context);
    let bootstrapingStartTime = getFutureTimestamp(10);
    let startTime = getFutureTimestamp(30);
    let endTime = getFutureTimestamp(60
    );

    logger.log(1, 'Creating a new token')
    let denom = await tokenFactory.createTokenFactoryToken(context, "creator", `stream_swap_test_token${Math.floor(Math.random() * 1000)}`)
    context.updateCreatedDenom(denom)
    // Hard coded stream out amount
    let streamAmount = 1000000000000000;

    let totalAmount = (streamAmount).toString();

    await tokenFactory.mintTokenFactoryToken(context, "creator", denom, totalAmount)
    let out_asset = { amount: streamAmount.toString(), denom };

    await controller.CreateStream(context, out_asset, deploymentConfig.in_denom, startTime.toString(), endTime.toString(), bootstrapingStartTime.toString(), { threshold: '1000000000000000' });

    // Await until bootstraping starts
    await sleepUntil(nanoToDate(bootstrapingStartTime));

    // Subsriber1 subscribes to the stream
    await stream.SubscribeToStream(context, { denom: deploymentConfig.in_denom, amount: '5000000' }, 'subscriber_1');

    // Await until stream starts
    await sleepUntil(nanoToDate(startTime));

    // Subsriber2 subscribes to the stream
    await stream.SubscribeToStream(context, { denom: deploymentConfig.in_denom, amount: '5000000' }, 'subscriber_2');

    // Await until stream ends
    await sleepUntil(nanoToDate(endTime));

    // Subsriber1 exits canceled stream
    await stream.ExitCanceled(context, 'subscriber_1');

    // Creator cancels the stream
    await stream.CancelStreamWithThreshold(context);

    // Subsriber2 exits canceled stream
    await stream.ExitCanceled(context, 'subscriber_2');
    await stream.QueryStream(context);

    await stream.SynchPosition(context, 'subscriber_1');
    await stream.SynchPosition(context, 'subscriber_2');

    await context.updateSimulationWithBalances();

    logger.log(1, 'Simulation 3 completed');
    context.collectSimulationEvents();
    let simulationResults = context.getSimulationResults();
    logger.log(1, `Simulation results: ${JSON.stringify(simulationResults, null, 2)}`);
}

simulation_3()
