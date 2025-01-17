/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { Coin } from "@cosmjs/amino";
import { MsgExecuteContractEncodeObject } from "cosmwasm";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { Group, Admin, Binary, InstantiateMsg, ContractInstantiateMsg, ExecuteMsg, QueryMsg, AdminResponse, Addr, MemberListResponse, Member, MemberResponse } from "./Splits.types";
export interface SplitsMessage {
  contractAddress: string;
  sender: string;
  updateAdmin: ({
    admin
  }: {
    admin?: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  distribute: (funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class SplitsMessageComposer implements SplitsMessage {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.updateAdmin = this.updateAdmin.bind(this);
    this.distribute = this.distribute.bind(this);
  }

  updateAdmin = ({
    admin
  }: {
    admin?: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_admin: {
            admin
          }
        })),
        funds
      })
    };
  };
  distribute = (funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          distribute: {}
        })),
        funds
      })
    };
  };
}