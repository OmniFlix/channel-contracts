import Context from '../helpers/context';
import Controller from '../helpers/channel.helper';
import TokenFactory from '../helpers/onft.helper';
import Stream from '../helpers/stream.helper';
import { logger } from '../utils/logger';
import { deploymentConfig } from '../../configs/deployment_config';
import { sleepUntil } from '../utils/time';
import { getFutureTimestamp, nanoToDate } from '../utils/datetime'


const simulation_2 = async () => {
    let context = new Context;
    let controller = new Controller;
    let stream = new Stream;
    let tokenFactory = new TokenFactory;

    await context.initialize();
    // Simulation 2: 
    // Create a stream with vesting

    // Details:
    // 1. Create a stream with vesting
    // 1.1 Vesting Schedule: saturating_linear
    // 1.2. Vesting duration is 120 seconds.
    // 1.3. Stream waiting duration is 30 seconds.
    // 1.4. Stream bootstraping duration is 60 seconds.
    // 1.5. Stream duration is 120 seconds.

    // 2. Subscribe to the stream at bootstraping. <Subsriber1>
    // 2.1. Subsriber1 subscribes with 5_000_000 amount.
    // 3. Subscriber1 withdraws full amount from the stream in bootstraping.
    // 4. Synch subsriber1 position.
    // 5. Subscribe to the stream at bootstraping. <Subsriber2>
    // 6. Subscriber2 increase subsription amount at start+30 seconds.
    // 7. Subscriber2 withdraws half amount from the stream at start+60 seconds.
    // 8. Subsriber1 Subsribes to the stream at start+60 seconds.
    // 9. Subsriber1 exits the stream after it ends.
    // 9.1. Subsriber2 exits the stream after it ends.
    // 10. Creator finalizes the stream after it ends.
    await controller.InstantiateStreamSwapController(context);
    let bootstrapingStartTime = getFutureTimestamp(30);
    let startTime = getFutureTimestamp(90);
    let endTime = getFutureTimestamp(210);

    logger.log(1, 'Creating a new token')
    let denom = await tokenFactory.createTokenFactoryToken(context, "creator", `stream_swap_test_token${Math.floor(Math.random() * 1000)}`)
    context.updateCreatedDenom(denom)
    // Hard coded stream out amount
    let streamAmount = 1000000000000000;

    let totalAmount = (streamAmount).toString();

    await tokenFactory.mintTokenFactoryToken(context, "creator", denom, totalAmount)
    let out_asset = { amount: streamAmount.toString(), denom };

    await controller.CreateStream(context, out_asset, deploymentConfig.in_denom, startTime.toString(), endTime.toString(), bootstrapingStartTime.toString(), { with_vesting: true });

    // Await until bootstraping starts
    await sleepUntil(nanoToDate(bootstrapingStartTime));

    // Subsriber1 subscribes to the stream
    await stream.SubscribeToStream(context, { denom: deploymentConfig.in_denom, amount: '5000000' }, 'subscriber_1');

    // Withdraw full amount from the stream
    await stream.WithdrawFromStream(context, '5000000', 'subscriber_1');

    // Synch subsriber1 position
    await stream.SynchPosition(context, 'subscriber_1');

    // Subsriber2 subscribes to the stream
    await stream.SubscribeToStream(context, { denom: deploymentConfig.in_denom, amount: '50000' }, 'subscriber_2');

    // Await until start time
    await sleepUntil(nanoToDate(startTime));

    // Increase subsription amount
    await stream.SubscribeToStream(context, { denom: deploymentConfig.in_denom, amount: '100000' }, 'subscriber_2');

    // Await until start+60 seconds
    await sleepUntil(nanoToDate(startTime + 60));

    // Withdraw half amount from the stream
    await stream.WithdrawFromStream(context, '50000', 'subscriber_2');

    // Subsriber1 Subsribes to the stream
    await stream.SubscribeToStream(context, { denom: deploymentConfig.in_denom, amount: '5000000' }, 'subscriber_1');

    // Await until end time
    await sleepUntil(nanoToDate(endTime));

    // Subsriber1 exits the stream
    await stream.ExitStream(context, 'subscriber_1');

    // Subsriber2 exits the stream
    await stream.ExitStream(context, 'subscriber_2');

    // Creator finalizes the stream
    await stream.FinalizeStream(context, undefined);

    // Synch subsriber1 position
    await stream.SynchPosition(context, 'subscriber_1');
    // Synch subsriber2 position
    await stream.SynchPosition(context, 'subscriber_2');
    // 
    await stream.QueryStream(context);

    await context.updateSimulationWithBalances();

    logger.log(1, 'Simulation 2 completed');
    context.collectSimulationEvents();
    let simulationResults = context.getSimulationResults();
    logger.log(1, `Simulation results: ${JSON.stringify(simulationResults, null, 2)}`);
}

simulation_2()
