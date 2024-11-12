
import { Field, Type } from 'protobufjs';
import { logger } from '../utils/logger';
import Context from './context';
import { MsgGrant, MsgExec } from 'cosmjs-types/cosmos/authz/v1beta1/tx'
import { GenericAuthorization, Grant } from 'cosmjs-types/cosmos/authz/v1beta1/authz';
import { EncodeObject } from '@cosmjs/proto-signing';
import { MsgExecuteContract, MsgExecuteContractResponse } from 'cosmjs-types/cosmwasm/wasm/v1/tx';
import { deploymentConfig } from '../../configs/deployment_config.example';
import { chain } from 'lodash';
const encoding_1 = require("@cosmjs/encoding");
import chainConfig from '../../configs/chain_config.json'
import { toUtf8 } from '@cosmjs/encoding';
import { channel } from 'diagnostics_channel';

export default class AuthzHelper {
    giveAuthorization = async (context: Context, sender: string, grantee: string) => {
        const { client: senderClient, address: senderAddress } = context.getTestUser(sender);
        const { address: granteeAddress } = context.getTestUser(grantee);
        let grant: Grant = {
            authorization: {
                typeUrl: '/cosmos.authz.v1beta1.GenericAuthorization',
                value: GenericAuthorization.encode(
                    GenericAuthorization.fromPartial({
                        msg: "/cosmwasm.wasm.v1.MsgExecuteContract",
                    }),
                ).finish(),
            },
            expiration: undefined
        }

        interface MsgGrantAuthorizationEncodeObject extends EncodeObject {
            readonly typeUrl: '/cosmos.authz.v1beta1.MsgGrant';
            readonly value: Partial<MsgGrant>;
        }

        let msgGrant = MsgGrant.fromPartial({
            granter: senderAddress,
            grantee: granteeAddress,
            grant: grant
        });

        let msgGrantAuthorizationExecute: MsgGrantAuthorizationEncodeObject = {
            typeUrl: '/cosmos.authz.v1beta1.MsgGrant',
            value: msgGrant
        }

        let result = await senderClient.signAndBroadcast(senderAddress, [msgGrantAuthorizationExecute], "auto");

        logger.log(1, `Grant authorization from ${senderAddress} to ${granteeAddress} with result: ${result.transactionHash}`);
    }

    executeMsgWithAuthz = async (context: Context, sender: string, grantee: string, contract_address: string, amount: string, msg: any) => {
        const { address: granteeAddress, client: granteeClient } = context.getTestUser(grantee);
        const { address: senderAddress } = context.getTestUser(sender);
        interface MsgExecAllowanceEncodeObject extends EncodeObject {
            readonly typeUrl: "/cosmos.authz.v1beta1.MsgExec";
            readonly value: Partial<MsgExec>;
        }

        const authzExecuteContractMsg: MsgExecAllowanceEncodeObject = {
            typeUrl: "/cosmos.authz.v1beta1.MsgExec",
            value: MsgExec.fromPartial({
                grantee: granteeAddress,
                msgs: [{
                    typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
                    value: MsgExecuteContract.encode({
                        sender: senderAddress,
                        contract: contract_address,
                        msg: toUtf8(JSON.stringify(
                            msg)),
                        funds: [],
                    }).finish(),
                }],
            }),
        }

        let result = await granteeClient.signAndBroadcast(granteeAddress, [authzExecuteContractMsg], "auto");
        logger.log(1, `Executed msg with authz from ${granteeAddress} to ${contract_address} with amount with result: ${result.transactionHash}`);
    }
}
