// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,reason-string

import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { IAccessManager } from "@openzeppelin-contracts/access/manager/IAccessManager.sol";

abstract contract DeployAccessManagerWithRoles {
    function accessManagerSetTargetRoles(
        IAccessManager accessManager,
        address ics26,
        address ics20,
        address ics27Gmp,
        bool pubRelay
    )
        public
    {
        accessManager.setTargetFunctionRole(
            ics26, IBCRolesLib.ics26IdCustomizerSelectors(), IBCRolesLib.ID_CUSTOMIZER_ROLE
        );
        accessManager.setTargetFunctionRole(ics26, IBCRolesLib.ics26RelayerSelectors(), IBCRolesLib.RELAYER_ROLE);
        accessManager.setTargetFunctionRole(ics20, IBCRolesLib.pauserSelectors(), IBCRolesLib.PAUSER_ROLE);
        accessManager.setTargetFunctionRole(ics20, IBCRolesLib.unpauserSelectors(), IBCRolesLib.UNPAUSER_ROLE);
        accessManager.setTargetFunctionRole(
            ics20, IBCRolesLib.erc20CustomizerSelectors(), IBCRolesLib.ERC20_CUSTOMIZER_ROLE
        );
        accessManager.setTargetFunctionRole(
            ics20, IBCRolesLib.delegateSenderSelectors(), IBCRolesLib.DELEGATE_SENDER_ROLE
        );
        accessManager.setTargetFunctionRole(ics27Gmp, IBCRolesLib.pauserSelectors(), IBCRolesLib.PAUSER_ROLE);
        accessManager.setTargetFunctionRole(ics27Gmp, IBCRolesLib.unpauserSelectors(), IBCRolesLib.UNPAUSER_ROLE);
        // TODO: fix rate limiter role (#559)

        // Explicitly register admin-gated selectors so deployments expose the full role map.
        accessManager.setTargetFunctionRole(ics26, IBCRolesLib.ics26ClientMigrationSelectors(), IBCRolesLib.ADMIN_ROLE);
        accessManager.setTargetFunctionRole(ics20, IBCRolesLib.ics20BeaconUpgradeSelectors(), IBCRolesLib.ADMIN_ROLE);
        accessManager.setTargetFunctionRole(ics27Gmp, IBCRolesLib.ics27BeaconUpgradeSelectors(), IBCRolesLib.ADMIN_ROLE);
        accessManager.setTargetFunctionRole(ics20, IBCRolesLib.uupsUpgradeSelectors(), IBCRolesLib.ADMIN_ROLE);
        accessManager.setTargetFunctionRole(ics26, IBCRolesLib.uupsUpgradeSelectors(), IBCRolesLib.ADMIN_ROLE);
        accessManager.setTargetFunctionRole(ics27Gmp, IBCRolesLib.uupsUpgradeSelectors(), IBCRolesLib.ADMIN_ROLE);

        if (pubRelay) {
            accessManager.setTargetFunctionRole(ics26, IBCRolesLib.ics26RelayerSelectors(), IBCRolesLib.PUBLIC_ROLE);
        }
    }

    function accessManagerSetRoles(
        IAccessManager accessManager,
        address[] memory relayers,
        address[] memory pausers,
        address[] memory unpausers,
        address idCustomizer,
        address erc20Customizer,
        address delegateSender
    )
        public
    {
        for (uint256 i = 0; i < relayers.length; ++i) {
            accessManager.grantRole(IBCRolesLib.RELAYER_ROLE, relayers[i], 0);
        }
        for (uint256 i = 0; i < pausers.length; ++i) {
            accessManager.grantRole(IBCRolesLib.PAUSER_ROLE, pausers[i], 0);
        }
        for (uint256 i = 0; i < unpausers.length; ++i) {
            accessManager.grantRole(IBCRolesLib.UNPAUSER_ROLE, unpausers[i], 0);
        }
        if (idCustomizer != address(0)) {
            accessManager.grantRole(IBCRolesLib.ID_CUSTOMIZER_ROLE, idCustomizer, 0);
        }
        if (erc20Customizer != address(0)) {
            accessManager.grantRole(IBCRolesLib.ERC20_CUSTOMIZER_ROLE, erc20Customizer, 0);
        }
        if (delegateSender != address(0)) {
            accessManager.grantRole(IBCRolesLib.DELEGATE_SENDER_ROLE, delegateSender, 0);
        }
    }
}
