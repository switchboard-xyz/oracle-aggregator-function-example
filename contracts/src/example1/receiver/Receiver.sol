pragma solidity ^0.8.9;

import {ReceiverLib} from "./ReceiverLib.sol";
import {AdminLib} from "../admin/AdminLib.sol";
import {ErrorLib} from "../error/ErrorLib.sol";
import "@pythnetwork/pyth-sdk-solidity/PythStructs.sol";

import {Switchboard} from "@switchboard-xyz/evm.js/contracts/arbitrum/testnet/Switchboard.sol";

contract Receiver {

    function callback(
        uint256[] calldata switchboardPrices,
        address[] calldata chainlinkPriceIds,
        bytes32[] calldata pythPriceIds,
        bytes[] calldata pythVaas
    ) external payable {
        address functionId = Switchboard.getEncodedFunctionId();
        if (AdminLib.functionId() == address(0)) {
            AdminLib.setFunctionId(functionId);
        }

        // Assert that the sender is switchboard & the correct function id is encoded
        if (functionId != AdminLib.functionId()) {
            revert ErrorLib.InvalidSender(AdminLib.functionId(), functionId);
        }
        ReceiverLib.callback(switchboardPrices, chainlinkPriceIds, pythPriceIds, pythVaas);
    }

    function viewData() external view returns (uint256 switchboardPrice, uint256 chainlinkPrice, uint256 pythPrice, uint256 answer, uint256 variance) {
        return ReceiverLib.viewData();
    }
}
