// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package ics26router

import (
	"context"
	"errors"
	"math/big"
	"strings"
	"time"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/event"
)

// Reference imports to suppress errors if they are not otherwise used.
var (
	_ = errors.New
	_ = big.NewInt
	_ = strings.NewReader
	_ = ethereum.NotFound
	_ = bind.Bind
	_ = common.Big1
	_ = types.BloomLookup
	_ = event.NewSubscription
	_ = abi.ConvertType
	_ = time.Tick
	_ = context.Background
)

// IICS02ClientMsgsCounterpartyInfo is an auto generated low-level Go binding around an user-defined struct.
type IICS02ClientMsgsCounterpartyInfo struct {
	ClientId     string
	MerklePrefix [][]byte
}

// IICS02ClientMsgsHeight is an auto generated low-level Go binding around an user-defined struct.
type IICS02ClientMsgsHeight struct {
	RevisionNumber uint64
	RevisionHeight uint64
}

// IICS26RouterMsgsMsgAckPacket is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsMsgAckPacket struct {
	Packet          IICS26RouterMsgsPacket
	Acknowledgement []byte
	ProofAcked      []byte
	ProofHeight     IICS02ClientMsgsHeight
}

// IICS26RouterMsgsMsgRecvPacket is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsMsgRecvPacket struct {
	Packet          IICS26RouterMsgsPacket
	ProofCommitment []byte
	ProofHeight     IICS02ClientMsgsHeight
}

// IICS26RouterMsgsMsgSendPacket is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsMsgSendPacket struct {
	SourceClient     string
	TimeoutTimestamp uint64
	Payload          IICS26RouterMsgsPayload
}

// IICS26RouterMsgsMsgTimeoutPacket is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsMsgTimeoutPacket struct {
	Packet       IICS26RouterMsgsPacket
	ProofTimeout []byte
	ProofHeight  IICS02ClientMsgsHeight
}

// IICS26RouterMsgsPacket is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsPacket struct {
	Sequence         uint64
	SourceClient     string
	DestClient       string
	TimeoutTimestamp uint64
	Payloads         []IICS26RouterMsgsPayload
}

// IICS26RouterMsgsPayload is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsPayload struct {
	SourcePort string
	DestPort   string
	Version    string
	Encoding   string
	Value      []byte
}

// ILightClientMsgsMsgVerifyMembership is an auto generated low-level Go binding around an user-defined struct.
type ILightClientMsgsMsgVerifyMembership struct {
	Proof       []byte
	ProofHeight IICS02ClientMsgsHeight
	Path        [][]byte
	Value       []byte
}

// ILightClientMsgsMsgVerifyNonMembership is an auto generated low-level Go binding around an user-defined struct.
type ILightClientMsgsMsgVerifyNonMembership struct {
	Proof       []byte
	ProofHeight IICS02ClientMsgsHeight
	Path        [][]byte
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ackPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.MsgAckPacket\",\"components\":[{\"name\":\"packet\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofAcked\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"addClient\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyInfo\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.CounterpartyInfo\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"merklePrefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]},{\"name\":\"client\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"addClient\",\"inputs\":[{\"name\":\"counterpartyInfo\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.CounterpartyInfo\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"merklePrefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]},{\"name\":\"client\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"addIBCApp\",\"inputs\":[{\"name\":\"app\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"addIBCApp\",\"inputs\":[{\"name\":\"portId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"app\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"authority\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getClient\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractILightClient\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCommitment\",\"inputs\":[{\"name\":\"hashedPath\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCounterparty\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.CounterpartyInfo\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"merklePrefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIBCApp\",\"inputs\":[{\"name\":\"portId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIIBCApp\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getNextClientSeq\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"isConsumingScheduledOp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"migrateClient\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyInfo\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.CounterpartyInfo\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"merklePrefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]},{\"name\":\"client\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"multicall\",\"inputs\":[{\"name\":\"data\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"outputs\":[{\"name\":\"results\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"recvPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.MsgRecvPacket\",\"components\":[{\"name\":\"packet\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"proofCommitment\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"sendPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.MsgSendPacket\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setAuthority\",\"inputs\":[{\"name\":\"newAuthority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"submitMisbehaviour\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"misbehaviourMsg\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"timeoutPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.MsgTimeoutPacket\",\"components\":[{\"name\":\"packet\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"proofTimeout\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"updateMsg\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"enumILightClientMsgs.UpdateResult\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"verifyMembership\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"verifyNonMembership\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyNonMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"AckPacket\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":true,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"packet\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"indexed\":false,\"internalType\":\"bytes\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"AuthorityUpdated\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IBCAppAdded\",\"inputs\":[{\"name\":\"portId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"app\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IBCAppRecvPacketCallbackError\",\"inputs\":[{\"name\":\"reason\",\"type\":\"bytes\",\"indexed\":false,\"internalType\":\"bytes\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ICS02ClientAdded\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"counterpartyInfo\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structIICS02ClientMsgs.CounterpartyInfo\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"merklePrefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]},{\"name\":\"client\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ICS02ClientMigrated\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"counterpartyInfo\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structIICS02ClientMsgs.CounterpartyInfo\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"merklePrefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]},{\"name\":\"client\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ICS02ClientUpdated\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"result\",\"type\":\"uint8\",\"indexed\":false,\"internalType\":\"enumILightClientMsgs.UpdateResult\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ICS02MisbehaviourSubmitted\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Noop\",\"inputs\":[],\"anonymous\":false},{\"type\":\"event\",\"name\":\"SendPacket\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":true,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"packet\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"TimeoutPacket\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":true,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"packet\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"WriteAcknowledgement\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":true,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"packet\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"acknowledgements\",\"type\":\"bytes[]\",\"indexed\":false,\"internalType\":\"bytes[]\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AccessManagedInvalidAuthority\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AccessManagedRequiredDelay\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"delay\",\"type\":\"uint32\",\"internalType\":\"uint32\"}]},{\"type\":\"error\",\"name\":\"AccessManagedUnauthorized\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"DefaultAdminRoleCannotBeGranted\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IBCAppNotFound\",\"inputs\":[{\"name\":\"portId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCAsyncAcknowledgementNotSupported\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IBCClientAlreadyExists\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCClientNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCCounterpartyClientNotFound\",\"inputs\":[{\"name\":\"counterpartyClientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCErrorUniversalAcknowledgement\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IBCFailedCallback\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IBCInvalidClientId\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCInvalidCounterparty\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCInvalidPortIdentifier\",\"inputs\":[{\"name\":\"portId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCInvalidTimeoutDuration\",\"inputs\":[{\"name\":\"maxTimeoutDuration\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"actualTimeoutDuration\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"IBCInvalidTimeoutTimestamp\",\"inputs\":[{\"name\":\"timeoutTimestamp\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"comparedTimestamp\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"IBCMultiPayloadPacketNotSupported\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IBCPacketAcknowledgementAlreadyExists\",\"inputs\":[{\"name\":\"path\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"IBCPacketCommitmentAlreadyExists\",\"inputs\":[{\"name\":\"path\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"IBCPacketCommitmentMismatch\",\"inputs\":[{\"name\":\"expected\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"actual\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"IBCPacketReceiptMismatch\",\"inputs\":[{\"name\":\"expected\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"actual\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"IBCPortAlreadyExists\",\"inputs\":[{\"name\":\"portId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IBCUnauthorizedSender\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidMerklePrefix\",\"inputs\":[{\"name\":\"prefix\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]},{\"type\":\"error\",\"name\":\"NoAcknowledgements\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"StringsInsufficientHexLength\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"Unreachable\",\"inputs\":[]}]",
	Bin: "0x60a080604052346100c257306080525f5160206156375f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b60405161557090816100c7823960805181818161133a01526113fe0152f35b6001600160401b0319166001600160401b039081175f5160206156375f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe60806040526004361015610011575f80fd5b5f5f3560e01c80631bca011a14611dea5780631ec43e2314611d8f5780632447af2914611d5957806327f146f314611d1d5780634481f4d214611c1b5780634b720d5b14611ac35780634d6e7ce3146116825780634f1ef286146113b257806352d1902d1461131357806355acb6191461119c5780635ebd10ca146111745780635f516889146110a35780636fbf807914610f795780637795820c14610f305780637a9e5e4b14610e595780637eb7893214610dff5780638fb3603714610d6c5780639e2e5c8314610c5b578063ac9650d814610b13578063ad3cb1cc14610ab2578063b0777bfa14610a3f578063b98c330a146109f0578063bf7e214f1461099d578063c4d66de8146107ca578063cce0b265146103f15763e3cb36a014610138575f80fd5b346103ee5760406003193601126103ee576004359067ffffffffffffffff82116103ee57604060031983360301126103ee57610172611e8a565b9161017d3633613f0d565b6101856144e9565b927f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a44960254915f1983146103c157600183017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a449602558383807a184f03e93ff9f4daa797ed6e38ed64bf6a1f010000000000000000811015610396575b50806d04ee2d6d415b85acef8100000000600a92101561037b575b662386f26fc10000811015610367575b6305f5e100811015610356575b612710811015610347575b6064811015610339575b1015610331575b6001810193600a5f19602161027f61026989611fed565b986102776040519a8b611fca565b808a52611fed565b94601f1960208a0196013687378801015b01917f30313233343536373839616263646566000000000000000000000000000000008282061a8353049081156102cc57600a905f1990610290565b505061032d9560209586610310936103199760405199858b9651918291018588015e85019083820190858252519283915e010190815203601f198101865285611fca565b600401836146f2565b604051918291602083526020830190611f24565b0390f35b600101610252565b60646002910492019161024b565b61271060049104920191610241565b6305f5e10060089104920191610236565b662386f26fc1000060109104920191610229565b6d04ee2d6d415b85acef810000000060209104920191610219565b604092507a184f03e93ff9f4daa797ed6e38ed64bf6a1f01000000000000000090049050600a6101fe565b6024847f4e487b710000000000000000000000000000000000000000000000000000000081526011600452fd5b80fd5b50346103ee5761040036611ead565b92919061040d3633613f0d565b610417828461371b565b506104228284612cb5565b61042c82806121fb565b9067ffffffffffffffff8211610737576104508261044a85546137af565b85613e0f565b8790601f831160011461076457918061048192600195948b92610618575b50505f198260011b9260031b1c19161790565b81555b0161049260208301836121a7565b916801000000000000000083116107375780548382558084106106b9575b508752602087208791805b8484106105a657505050505061059a73ffffffffffffffffffffffffffffffffffffffff7f23c2e29d6ae84e79fa116b8afd6e28ddc1de7f473d3edb407fbd08093c3ed6bf951691604051848682376020818681017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a44960081520301902073ffffffffffffffffffffffffffffffffffffffff84167fffffffffffffffffffffffff000000000000000000000000000000000000000082541617905561058c60405195869560608752606087019161224c565b908482036020860152613e5e565b9060408301520390a180f35b6105b081836121fb565b9067ffffffffffffffff821161068c576105d4826105ce87546137af565b87613e0f565b8b908c601f841160011461062357836001959294602094879661060994926106185750505f198260011b9260031b1c19161790565b86555b019301930192916104bb565b013590505f8061046e565b91601f19841687845260208420935b818110610674575093602093600196938796938388951061065b575b505050811b01865561060c565b5f1960f88560031b161c199101351690555f808061064e565b91936020600181928787013581550195019201610632565b60248c7f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b818952838060208b2001910390895b8281106106d65750506104b0565b808b60019284016106e781546137af565b806106f6575b505050016106c8565b601f8111851461070b5750555b8b5f806106ed565b8183526020832061072791601f0160051c5f1901908601613df4565b8082528160208120915555610703565b6024887f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b8389526020892091601f1984168a5b8181106107b25750916001959492918387959310610799575b505050811b018155610484565b5f1960f88560031b161c199101351690555f808061078c565b91936020600181928787013581550195019201610773565b50346103ee5760206003193601126103ee576107e4611e67565b7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005467ffffffffffffffff811690816109755760401c60ff16908115610969575b50610941577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0080547fffffffffffffffffffffffffffffffffffffffffffffff00000000000000000016680100000000000000021790556108ad90610888615339565b610890615339565b610898615339565b6108a0615339565b6108a8615339565b615013565b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160028152a180f35b6004827ff92ee8a9000000000000000000000000000000000000000000000000000000008152fd5b6002915010155f610825565b6004847ff92ee8a9000000000000000000000000000000000000000000000000000000008152fd5b50346103ee57806003193601126103ee57602073ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416604051908152f35b50346103ee57610a19610a023661205d565b610a0a613e99565b610a143633613f0d565b613a34565b807f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d80f35b50346103ee5760206003193601126103ee576004359067ffffffffffffffff82116103ee5761032d610a7d610a773660048601611e39565b90613800565b604051918291602083526020610a9e82516040838701526060860190611f24565b910151601f1984830301604085015261211c565b50346103ee57806003193601126103ee575061032d604051610ad5604082611fca565b600581527f352e302e300000000000000000000000000000000000000000000000000000006020820152604051918291602083526020830190611f24565b50346103ee5760206003193601126103ee576004359067ffffffffffffffff82116103ee57366023830112156103ee5781600401359067ffffffffffffffff82116103ee5760248301903660248460051b860101116103ee57604051906020610b7c8184611fca565b81835280830193601f198201368637610b94866122e3565b96610ba26040519889611fca565b868852601f19610bb1886122e3565b0183855b828110610c4b57505050835b87811015610c3857600190610c1c610c16610be460248460051b870101876121fb565b9190888c8c8c8660405197889686880137850191848301918252519283915e01018a815203601f198101835282611fca565b306150be565b610c26828c612342565b52610c31818b612342565b5001610bc1565b6040518481528061032d8187018c61211c565b606082828d010152018490610bb5565b5034610d6857610c6a36612090565b73ffffffffffffffffffffffffffffffffffffffff610c8c848695979661371b565b1691823b15610d6857610cd9925f92836040518096819582947fddba653700000000000000000000000000000000000000000000000000000000845260206004850152602484019161224c565b03925af18015610d5d57610d29575b507fa263f0a976b2937a51fd2e416491cf0ca724d5499fa870715929dfde4ee4a4309192610d2360405192839260208452602084019161224c565b0390a180f35b7fa263f0a976b2937a51fd2e416491cf0ca724d5499fa870715929dfde4ee4a43092505f610d5691611fca565b5f91610ce8565b6040513d5f823e3d90fd5b5f80fd5b34610d68575f600319360112610d68577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005460a01c60ff1615610df75760207f8fb36037000000000000000000000000000000000000000000000000000000005b7fffffffff0000000000000000000000000000000000000000000000000000000060405191168152f35b60205f610dcd565b34610d68576020600319360112610d685760043567ffffffffffffffff8111610d6857610e3b610e356020923690600401611e39565b9061371b565b73ffffffffffffffffffffffffffffffffffffffff60405191168152f35b34610d68576020600319360112610d6857610e72611e67565b73ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054163303610f0457803b15610ec257610ec090615013565b005b73ffffffffffffffffffffffffffffffffffffffff907fc2f31e5e000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b7f068ca9d8000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b34610d68576020600319360112610d68576004355f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600602052602060405f2054604051908152f35b34610d68576020610ff8610f8c36612090565b610f9a959293953633613f0d565b73ffffffffffffffffffffffffffffffffffffffff610fb9858861371b565b16905f6040518097819582947f0bece356000000000000000000000000000000000000000000000000000000008452886004850152602484019161224c565b03925af1918215610d5d575f92611065575b506020927f87bbef2779889a19f0435ddca81fda94132c06ffddb0ea73def256307a293aef9161104760405192839260408452604084019161224c565b611053868301866120e2565b0390a161106360405180926120e2565bf35b9091506020813d60201161109b575b8161108160209383611fca565b81010312610d6857516003811015610d685790602061100a565b3d9150611074565b34610d68576040600319360112610d685760043567ffffffffffffffff8111610d685761114a6110da61114f923690600401611e39565b91906110e4611e8a565b926110ed613e99565b6110f73633613f0d565b61110481838115156136d2565b611126818361111f611117368484612009565b805190615276565b50156136d2565b611143818361113e611139368484612009565b614535565b6136d2565b3691612009565b614aee565b5f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b34610d685761114f6111853661205d565b61118d613e99565b6111973633613f0d565b6130e2565b34610d68576040600319360112610d685760043567ffffffffffffffff8111610d68576111cd903690600401611e39565b9060243567ffffffffffffffff8111610d68578060040160a06003198336030112610d68576112b25f73ffffffffffffffffffffffffffffffffffffffff6112246112c49760209761121f3633613f0d565b61371b565b1692604051968795869485937f682ed5f000000000000000000000000000000000000000000000000000000000855289600486015260846112aa61127c61126b8480612546565b60a060248b015260c48a019161224c565b61128c6044890160248801612d81565b6112996064870185612596565b906023198a840301858b0152612daa565b930190612546565b906023198584030160a486015261224c565b03925af18015610d5d575f906112e0575b602090604051908152f35b506020813d60201161130b575b816112fa60209383611fca565b81010312610d6857602090516112d5565b3d91506112ed565b34610d68575f600319360112610d685773ffffffffffffffffffffffffffffffffffffffff7f000000000000000000000000000000000000000000000000000000000000000016300361138a5760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b7fe07c8dba000000000000000000000000000000000000000000000000000000005f5260045ffd5b6040600319360112610d68576113c6611e67565b60243567ffffffffffffffff8111610d68576113e690369060040161203f565b9073ffffffffffffffffffffffffffffffffffffffff7f000000000000000000000000000000000000000000000000000000000000000016803014908115611640575b5061138a576114383633613f0d565b73ffffffffffffffffffffffffffffffffffffffff8116916040517f52d1902d000000000000000000000000000000000000000000000000000000008152602081600481875afa5f918161160c575b506114b857837f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b807f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8592036115e15750823b156115b657807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a280511561158557610ec0916150be565b50503461158e57005b7fb398979f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7faa1d49a4000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b9091506020813d602011611638575b8161162860209383611fca565b81010312610d6857519085611487565b3d915061161b565b905073ffffffffffffffffffffffffffffffffffffffff7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416141583611429565b34610d68576020600319360112610d685760043567ffffffffffffffff8111610d6857806004019060606003198236030112610d68576116c0613e99565b604481019173ffffffffffffffffffffffffffffffffffffffff6116f66116f06116ea8685612174565b806121fb565b90612ced565b163303611a9757602461170c610a7783806121fb565b5192019061173a61171c836122ce565b4290611727856122ce565b9067ffffffffffffffff42911611612e09565b6201518061175a4267ffffffffffffffff611754866122ce565b16612e4b565b116117714267ffffffffffffffff611754866122ce565b90611a655750602061178382806121fb565b919082604051938492833781017f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e07476018152030190209267ffffffffffffffff8454169467ffffffffffffffff8614611a385767ffffffffffffffff600161182097011694857fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000082541617905561181883806121fb565b9690946122ce565b604094855197611830878a611fca565b60018952601f1987015f5b818110611a025750509267ffffffffffffffff889993611871611899946118a8978b519d8e61186981611f49565b523691612009565b9660208c01978852898c01521660608a01528260808a0152611894369187612174565b612412565b6118a282612335565b52612335565b506118c0815167ffffffffffffffff87511690614c51565b6020815191012090815f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e074760060205261190b845f205415915167ffffffffffffffff88511690614c51565b90156119c15750937fab3a4458a269be61dfa43faa33aa7b1f5d570716f83ad078bc2ba5dab039abae611995611978869460209861194886614ccb565b905f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e07476008a52875f2055806121fb565b90818751928392833781015f815203902092855191829182612e58565b0390a35f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d51908152f35b6119fe9084519182917f91ffd924000000000000000000000000000000000000000000000000000000008352602060048401526024830190611f24565b0390fd5b8089602080938e6060845194611a1786611f49565b8186528185870152850152606080850152606060808501520101520161183b565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b7f715fed60000000000000000000000000000000000000000000000000000000005f526201518060045260245260445ffd5b7fbe2f2b45000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b34610d68576020600319360112610d6857611adc611e67565b611ae4613e99565b611aee3633613f0d565b73ffffffffffffffffffffffffffffffffffffffff81169081611b11602a611fed565b90611b1f6040519283611fca565b602a8252611b2d602a611fed565b601f19602084019101368237825115611bee5760309053815160011015611bee576078602183015360295b60018111611ba05750611b6f5761114f9250614aee565b827fe22e27eb000000000000000000000000000000000000000000000000000000005f52600452601460245260445ffd5b90600f81166010811015611bee577f3031323334353637383961626364656600000000000000000000000000000000901a611bdb8385614524565b5360041c908015611a38575f1901611b58565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b34610d68576040600319360112610d685760043567ffffffffffffffff8111610d6857611c4c903690600401611e39565b9060243567ffffffffffffffff8111610d68578060040160806003198336030112610d6857611d0b5f73ffffffffffffffffffffffffffffffffffffffff611c9e6112c49760209761121f3633613f0d565b1692604051968795869485937f4d6d9ffb0000000000000000000000000000000000000000000000000000000085528960048601526064611cf3611ce28380612546565b608060248a015260a489019161224c565b93611d046044880160248301612d81565b0190612596565b90602319858403016084860152612daa565b34610d68575f600319360112610d685760207f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a44960254604051908152f35b34610d68576020600319360112610d685760043567ffffffffffffffff8111610d6857610e3b6116f06020923690600401611e39565b34610d685761032d610319611143611da636611ead565b90611db5949293943633613f0d565b611dc28486811515612c6c565b611dda8486611dd5611139368484612009565b612c6c565b611de5368587612009565b6146f2565b34610d68576020600319360112610d685760043567ffffffffffffffff8111610d685760a06003198236030112610d685761114f90611e27613e99565b611e313633613f0d565b600401612786565b9181601f84011215610d685782359167ffffffffffffffff8311610d685760208381860195010111610d6857565b6004359073ffffffffffffffffffffffffffffffffffffffff82168203610d6857565b6024359073ffffffffffffffffffffffffffffffffffffffff82168203610d6857565b6060600319820112610d685760043567ffffffffffffffff8111610d685781611ed891600401611e39565b929092916024359067ffffffffffffffff8211610d685760031982604092030112610d68576004019060443573ffffffffffffffffffffffffffffffffffffffff81168103610d685790565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b60a0810190811067ffffffffffffffff821117611f6557604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b6040810190811067ffffffffffffffff821117611f6557604052565b6080810190811067ffffffffffffffff821117611f6557604052565b90601f601f19910116810190811067ffffffffffffffff821117611f6557604052565b67ffffffffffffffff8111611f6557601f01601f191660200190565b92919261201582611fed565b916120236040519384611fca565b829481845281830111610d68578281602093845f960137010152565b9080601f83011215610d685781602061205a93359101612009565b90565b6020600319820112610d68576004359067ffffffffffffffff8211610d685760031982608092030112610d685760040190565b6040600319820112610d685760043567ffffffffffffffff8111610d6857816120bb91600401611e39565b929092916024359067ffffffffffffffff8211610d68576120de91600401611e39565b9091565b9060038210156120ef5752565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602160045260245ffd5b9080602083519182815201916020808360051b8301019401925f915b83831061214757505050505090565b909192939460208061216583601f1986600196030187528951611f24565b97019301930191939290612138565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6181360301821215610d68570190565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe181360301821215610d68570180359067ffffffffffffffff8211610d6857602001918160051b36038313610d6857565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe181360301821215610d68570180359067ffffffffffffffff8211610d6857602001918136038313610d6857565b601f8260209493601f1993818652868601375f8582860101520116010190565b9290921561227957505050565b6122bc92916119fe916040519485947f9fff831f000000000000000000000000000000000000000000000000000000008652604060048701526044860190611f24565b9160031985840301602486015261224c565b3567ffffffffffffffff81168103610d685790565b67ffffffffffffffff8111611f655760051b60200190565b6040805190919061230c8382611fca565b6001815291601f1901825f5b82811061232457505050565b806060602080938501015201612318565b805115611bee5760200190565b8051821015611bee5760209160051b010190565b359067ffffffffffffffff82168203610d6857565b9190826040910312610d685760405161238381611f92565b602061239c81839561239481612356565b855201612356565b910152565b9061205a916020815260606123fd6123c5845160a0602086015260c0850190611f24565b602085810151805167ffffffffffffffff90811660408801529101511660608501526040850151601f1985830301608086015261211c565b9201519060a0601f1982850301910152611f24565b919060a083820312610d68576040519061242b82611f49565b8193803567ffffffffffffffff8111610d68578261244a91830161203f565b8352602081013567ffffffffffffffff8111610d68578261246c91830161203f565b6020840152604081013567ffffffffffffffff8111610d68578261249191830161203f565b6040840152606081013567ffffffffffffffff8111610d6857826124b691830161203f565b606084015260808101359167ffffffffffffffff8311610d685760809261239c920161203f565b61205a9160806125356125236125116124ff865160a0875260a0870190611f24565b60208701518682036020880152611f24565b60408601518582036040870152611f24565b60608501518482036060860152611f24565b920151906080818403910152611f24565b90357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe182360301811215610d6857016020813591019167ffffffffffffffff8211610d68578136038313610d6857565b90357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe182360301811215610d6857016020813591019167ffffffffffffffff8211610d68578160051b36038313610d6857565b9067ffffffffffffffff6125fc83612356565b1681526126676126416126266126156020860186612546565b60a0602087015260a086019161224c565b6126336040860186612546565b90858303604087015261224c565b9267ffffffffffffffff61265760608301612356565b1660608401526080810190612596565b9290916080818303910152828152602081019260208160051b83010193835f917fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6182360301945b8484106126bf575050505050505090565b90919293949596601f19828203018352873587811215610d68576020612775600193878394019061276761275c61274161272661270d6126ff8780612546565b60a0885260a088019161224c565b61271989880188612546565b908783038b89015261224c565b6127336040870187612546565b90868303604088015261224c565b61274e6060860186612546565b90858303606087015261224c565b926080810190612546565b91608081850391015261224c565b9901930194019291959493906126ae565b600161279f6127958380612174565b60808101906121a7565b905003612c44576127b36127958280612174565b15611bee576127c3818392612174565b5f6020612956816127e3610a776127da8880612174565b838101906121fb565b61282781518381519101206128086111436127fe8b80612174565b60408101906121fb565b8481519101201482519061281f6127fe8b80612174565b92909161226c565b61285b61285661283a6127fe8a80612174565b919061284e6128498c80612174565b6122ce565b923691612009565b614192565b906128d46128b06128988a61287f611143886128756122fb565b93019e8f906121fb565b61288882612335565b5261289281612335565b50614223565b936128a660408c018c6121fb565b9690940151614348565b9160405193878501528684526128c7604085611fca565b6040519461114386611fae565b83526128e33660608a0161236b565b858401526040830152606082015273ffffffffffffffffffffffffffffffffffffffff61291f610e356129168980612174565b868101906121fb565b16906040519485809481937f682ed5f0000000000000000000000000000000000000000000000000000000008352600483016123a1565b03925af18015610d5d57612c15575b506129786129738380612174565b614406565b15612bec5773ffffffffffffffffffffffffffffffffffffffff61299f6116f083806121fb565b166129b76129ad8480612174565b60208101906121fb565b6129c76127fe8680969496612174565b6129d76128498880959495612174565b916129e289896121fb565b9790916040519560c087019487861067ffffffffffffffff871117611f6557612a1a612a4394612a2394612a52986040523691612009565b88523691612009565b956020860196875267ffffffffffffffff60408701951685523690612412565b96606085019788523691612009565b6080830190815260a0830191338352853b15610d685760405196879586957f428e4e170000000000000000000000000000000000000000000000000000000087526004870160209052516024870160c0905260e48701612ab191611f24565b905190868103602319016044880152612ac991611f24565b915167ffffffffffffffff1660648601525190848103602319016084860152612af1916124dd565b9051908381036023190160a4850152612b0991611f24565b905173ffffffffffffffffffffffffffffffffffffffff1660c483015203815a5f948591f18015610d5d57612bd7927ff9bab74bcdb634f4d3dd064cc42a13df056598e1c0336905d2f5750fbfb08b7b92612bc792612bdc575b50612b716129ad8280612174565b949095612b95612b846128498580612174565b91612b8f8580612174565b946121fb565b96909781604051928392833781015f81520390209567ffffffffffffffff6040519586956040875260408701906125e9565b928584036020870152169761224c565b0390a3565b5f612be691611fca565b5f612b63565b5050507fd08bf58b0e4eec5bfc697a4fdbb6839057fbf4dd06f1b1ce07445c0e5a654caf5f80a1565b6020813d602011612c3c575b81612c2e60209383611fca565b81010312610d685751612965565b3d9150612c21565b7f356f4dbd000000000000000000000000000000000000000000000000000000005f5260045ffd5b91909115612c78575050565b6119fe6040519283927f4870bd7400000000000000000000000000000000000000000000000000000000845260206004850152602484019161224c565b60209082604051938492833781017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a44960181520301902090565b73ffffffffffffffffffffffffffffffffffffffff604051838382376020818581017fc5779f3c2c21083eefa6d04f6a698bc0d8c10db124ad5e0df6ef394b6d7bf6008152030190205416918215612d4457505090565b6119fe6040519283927fa09dbf5900000000000000000000000000000000000000000000000000000000845260206004850152602484019161224c565b67ffffffffffffffff612da46020809383612d9b82612356565b16865201612356565b16910152565b90602083828152019260208260051b82010193835f925b848410612dd15750505050505090565b909192939495602080612df983601f198660019603018852612df38b88612546565b9061224c565b9801940194019294939190612dc1565b15612e12575050565b67ffffffffffffffff907f65d30129000000000000000000000000000000000000000000000000000000005f521660045260245260445ffd5b91908203918211611a3857565b906020825267ffffffffffffffff81511660208301526080612ea2612e8c602084015160a0604087015260c0860190611f24565b6040840151601f19868303016060870152611f24565b9167ffffffffffffffff6060820151168285015201519160a0601f1982840301910152815180825260208201916020808360051b8301019401925f915b838310612eee57505050505090565b9091929394602080612f0c83601f19866001960301875289516124dd565b97019301930191939290612edf565b919060a083820312610d6857604051612f3381611f49565b8093612f3e81612356565b8252602081013567ffffffffffffffff8111610d685783612f6091830161203f565b6020830152604081013567ffffffffffffffff8111610d685783612f8591830161203f565b6040830152612f9660608201612356565b606083015260808101359067ffffffffffffffff8211610d6857019180601f84011215610d68578235612fc8816122e3565b93612fd66040519586611fca565b81855260208086019260051b82010191838311610d685760208201905b83821061300557505050505060800152565b813567ffffffffffffffff8111610d685760209161302887848094880101612412565b815201910190612ff3565b90608073ffffffffffffffffffffffffffffffffffffffff8161309d613077613065875160a0885260a0880190611f24565b60208801518782036020890152611f24565b67ffffffffffffffff6040880151166040870152606087015186820360608801526124dd565b9401511691015290565b604051906130b6604083611fca565b602082527f4774d4a575993f963b1c06573736617a457abef8589178db8d10c94b4ab511ab6020830152565b60016130f16127958380612174565b905003612c44576131056127958280612174565b15611bee578061311491612174565b905f60206132288161312c610a776127fe8780612174565b61315e81518381519101206131476111436129168a80612174565b8481519101201482519061281f6129168a80612174565b61318761317660606131708980612174565b016122ce565b429061172760606131708b80612174565b6131b76131b26131a361319a8980612174565b858101906121fb565b919061284e6128498b80612174565b614c51565b906131e66128b06131d96131d4366131cf8c80612174565b612f1b565b614ccb565b936128a6868b018b6121fb565b83526131f5366040890161236b565b858401526040830152606082015273ffffffffffffffffffffffffffffffffffffffff61291f610e356127fe8880612174565b03925af18015610d5d576136a3575b5061324a6132458280612174565b614f31565b1561367b575f8061332a61325c6122fb565b9467ffffffffffffffff6132e273ffffffffffffffffffffffffffffffffffffffff61328e6116f060208601866121fb565b169261329d6129ad8980612174565b93906132d08a612849612a1a6132c26132b96127fe8580612174565b93909480612174565b94604051996111438b611f49565b60208601521660408401523690612412565b60608201523360808201526040519485809481937f078c4a79000000000000000000000000000000000000000000000000000000008352602060048401526024830190613033565b03925af15f91816135ff575b5061357457503d1561356d573d61334c81611fed565b9061335a6040519283611fca565b81523d5f602083013e5b805115613545576133a47fb9edb487876e8be10f54e377c1a815a54ad92a6db1c9561dfe8fad2f0d1da84f91604051918291602083526020830190611f24565b0390a16133af6130a7565b6133b883612335565b526133c282612335565b505b6133ce8180612174565b6040810161343d61284e6128566133f66128566133eb86886121fb565b919061284e896122ce565b6020815191012094855f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e074760060205261343560405f20541595826121fb565b9390916122ce565b9015613507575061344d83614223565b905f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e074760060205260405f20557f76765590e2b799b0506100f8a6610cfecab2c71e8e1f8aa981b099aff0dfdb746134f7612bd76134ad6127fe8580612174565b6134c76134c06128498880969596612174565b9680612174565b9281604051928392833781015f81520390209467ffffffffffffffff6040519485946040865260408601906125e9565b918483036020860152169661211c565b6119fe906040519182917f40470d74000000000000000000000000000000000000000000000000000000008352602060048401526024830190611f24565b7fadef7fb8000000000000000000000000000000000000000000000000000000005f5260045ffd5b6060613364565b8051156135d7578051602082012061358a6130a7565b60208151910120146135af5761359f83612335565b526135a982612335565b506133c4565b7f6b2675e3000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fecfef798000000000000000000000000000000000000000000000000000000005f5260045ffd5b9091503d805f833e6136118183611fca565b810190602081830312610d685780519067ffffffffffffffff8211610d68570181601f82011215610d685780519061364882611fed565b926136566040519485611fca565b82845260208383010111610d6857815f9260208093018386015e83010152905f613336565b50507fd08bf58b0e4eec5bfc697a4fdbb6839057fbf4dd06f1b1ce07445c0e5a654caf5f80a1565b6020813d6020116136ca575b816136bc60209383611fca565b81010312610d685751613237565b3d91506136af565b919091156136de575050565b6119fe6040519283927f14d7124700000000000000000000000000000000000000000000000000000000845260206004850152602484019161224c565b73ffffffffffffffffffffffffffffffffffffffff604051838382376020818581017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a449600815203019020541691821561377257505090565b6119fe6040519283927fa0db16fe00000000000000000000000000000000000000000000000000000000845260206004850152602484019161224c565b90600182811c921680156137f6575b60208310146137c957565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f16916137be565b6060602060405161381081611f92565b828152015261381f8282612cb5565b916040519261382d84611f92565b6040515f825461383c816137af565b80845290600181169081156139f257506001146139af575b509061386581600194930382611fca565b855201805490613874826122e3565b916138826040519384611fca565b80835260208301915f5260205f20915f905b8282106138ee57505050506020840152825151156138b157505090565b6119fe6040519283927fdf95155a00000000000000000000000000000000000000000000000000000000845260206004850152602484019161224c565b6040515f85546138fd816137af565b808452906001811690811561396e5750600114613937575b506001928261392985946020940382611fca565b815201940191019092613894565b5f878152602081209092505b81831061395857505081016020016001613915565b6001816020925483868801015201920191613943565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208581019190915291151560051b8401909101915060019050613915565b5f8481526020812094939250905b8082106139d65750919250908101602001613865613854565b91929360018160209254838588010152019101909392916139bd565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208086019190915291151560051b840190910191506138659050613854565b6001613a436127958380612174565b905003612c4457613a576127958280612174565b15611bee5780613a6691612174565b906020613a79610a776127da8480612174565b613aab8151838151910120613a946111436127fe8780612174565b8481519101201482519061281f6127fe8780612174565b613aeb613ad5613ad0613ac16127fe8780612174565b919061284e6128498980612174565b615185565b613ae1848601866121fb565b9490930151614348565b916040516060810181811067ffffffffffffffff821117611f6557602093613bd693613b1b926040523691612009565b8152613bb45f613b2e366040890161236b565b9285810193845260408101968752613be873ffffffffffffffffffffffffffffffffffffffff613b6d610e35613b648c80612174565b8a8101906121fb565b1694604051988997889687957f4d6d9ffb0000000000000000000000000000000000000000000000000000000087528b6004880152516080602488015260a4870190611f24565b9251604486019067ffffffffffffffff60208092828151168552015116910152565b5160231984830301608485015261211c565b03925af18015610d5d575f90613dc0575b613c2f915067ffffffffffffffff613c1660606131708680612174565b16811015613c2960606131708680612174565b90612e09565b613c3c6129738280612174565b1561367b5781613cca73ffffffffffffffffffffffffffffffffffffffff613c706116f08467ffffffffffffffff976121fb565b1691613c7f6129ad8580612174565b9590613cb8613c916127fe8880612174565b613caf613ca16128498b80612174565b946040519b6111438d611f49565b8a523691612009565b60208801521660408601523690612412565b6060840152336080840152803b15610d6857613d215f939184926040519586809481937f5e32b6b6000000000000000000000000000000000000000000000000000000008352602060048401526024830190613033565b03925af1918215610d5d5767ffffffffffffffff92613db0575b507f01e5ed58494819ef3f6480dd08e433b7c08ed75c7abdf2c22c6f04b71340a168613d6a6129ad8380612174565b613d84613d7d6128498680989598612174565b9480612174565b9481604051928392833781015f815203902092612bd760405192839260208452169560208301906125e9565b5f613dba91611fca565b5f613d3b565b506020813d602011613dec575b81613dda60209383611fca565b81010312610d6857613c2f9051613bf9565b3d9150613dcd565b5f5b828110613e0257505050565b5f82820155600101613df6565b91601f8211613e1d57505050565b808211613e2957505050565b613e54925f5260205f20916020601f830160051c9210613e56575b601f82910160051c039101613df4565b565b5f9150613e44565b61205a91613e8b613e80613e728480612546565b60408552604085019161224c565b926020810190612596565b916020818503910152612daa565b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c613ee55760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00549173ffffffffffffffffffffffffffffffffffffffff83169281600411610d68575f5f9060405f81519673ffffffffffffffffffffffffffffffffffffffff60208901917fb700961300000000000000000000000000000000000000000000000000000000835216978860248201523060448201527fffffffff00000000000000000000000000000000000000000000000000000000833516606482015260648152613fdc608482611fca565b828052826020525190895afa61417f575b15613ffa575b5050505050565b63ffffffff1615614153577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055823b15610d68576020925f92836040518096819582947f94c7d7ee000000000000000000000000000000000000000000000000000000008452600484015260406024840152601f19601f6044850192808452808786860137868582860101520116010103925af18015610d5d57614143575b507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00555f80808080613ff3565b5f61414d91611fca565b5f6140d2565b827f068ca9d8000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b50505f516020518060201c150290613fed565b600961205a916020937fffffffffffffffff000000000000000000000000000000000000000000000000856040519687948051918291018387015e840101917f0300000000000000000000000000000000000000000000000000000000000000835260c01b1660018201520301601f198101835282611fca565b60209291908391805192839101825e019081520190565b908151156143205760209160405161423b8482611fca565b5f8152905f915b81518310156142a757845f816142588686612342565b51604051918183925191829101835e8101838152039060025afa15610d5d5760019061429f5f51916142916040519384928a840161420c565b03601f198101835282611fca565b920191614242565b90505f915092919260405161430060218286808201957f020000000000000000000000000000000000000000000000000000000000000087528051918291018484015e810186838201520301601f198101835282611fca565b604051918291518091835e8101838152039060025afa15610d5d575f5190565b7f760d6a9b000000000000000000000000000000000000000000000000000000005f5260045ffd5b908151156143cb5781515f198101908111611a3857602091828061436f6143a59487612342565b51926040519584879551918291018487015e8401908282015f8152815193849201905e01015f815203601f198101835282611fca565b81515f198101908111611a38576143c7916143c08285612342565b5282612342565b5090565b6040517fa7c34e4f00000000000000000000000000000000000000000000000000000000815260206004820152806119fe602482018561211c565b6144246131b261441960208401846121fb565b919061284e856122ce565b6020815191012090815f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e074760060205260405f205480156144e25761447b6131d46144716131d43686612f1b565b8314933690612f1b565b91156144b45750505f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e07476006020525f6040812055600190565b7f3f87a2ec000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b5050505f90565b604051906144f8604083611fca565b600782527f636c69656e742d000000000000000000000000000000000000000000000000006020830152565b908151811015611bee570160200190565b8051600481109081156146e7575b506146cb57614589604051614559604082611fca565b600881527f6368616e6e656c2d0000000000000000000000000000000000000000000000006020820152826151ff565b80156146d0575b6146cb575f5b81518110156146c4576145a98183614524565b5160f81c606181101590816146b8575b811561469a575b811561467c575b811561463d575b81156145e8575b506145e05750505f90565b600101614596565b6023811491508115614632575b8115614627575b811561461c575b8115614611575b505f6145d5565b603e9150145f61460a565b603c81149150614603565b605d811491506145fc565b605b811491506145f5565b9050602e81148015614672575b8015614668575b801561465e575b906145ce565b50602d8114614658565b50602b8114614651565b50605f811461464a565b905060418110158061468f575b906145c7565b50605a811115614689565b90506030811015806146ad575b906145c0565b5060398111156146a7565b607a81111591506145b9565b5050600190565b505f90565b506146e26146dc6144e9565b826151ff565b614590565b60809150115f614543565b91906040519173ffffffffffffffffffffffffffffffffffffffff845193602081818801968088835e81017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a4496008152030190205416614ab35773ffffffffffffffffffffffffffffffffffffffff169160405160208186518085835e81017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a44960081520301902073ffffffffffffffffffffffffffffffffffffffff84167fffffffffffffffffffffffff00000000000000000000000000000000000000008254161790556020604051809286518091835e81017f515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a44960181520301902061481482806121fb565b9067ffffffffffffffff8211611f65576148328261044a85546137af565b5f90601f8311600114614a4c57918061486292600195945f926106185750505f198260011b9260031b1c19161790565b81555b0161487360208301836121a7565b90680100000000000000008211611f655782548284558083106149ce575b505f928352602083209290805b8383106148f35750505050509161058c916148e87f0ecded31ecd211a73abf0fb3bc09150bbe321a05550fbe29ea0f16b6e25fbfa894604051948594606086526060860190611f24565b9060408301520390a1565b6148fd81836121fb565b9067ffffffffffffffff8211611f65576149218261491b89546137af565b89613e0f565b5f90601f83116001146149645792614955836001959460209487965f926106185750505f198260011b9260031b1c19161790565b88555b0195019201919361489e565b601f19831691885f5260205f20925f5b8181106149b6575093602093600196938796938388951061499d575b505050811b018855614958565b5f1960f88560031b161c199101351690555f8080614990565b91936020600181928787013581550195019201614974565b835f52828060205f20019103905f5b8281106149eb575050614891565b8060019183016149fb81546137af565b9081614a0a575b5050016149dd565b81601f5f93118514614a205750555b5f80614a02565b81835260208320614a3c91601f0160051c5f1901908601613df4565b8082528160208120915555614a19565b601f19831691845f5260205f20925f5b818110614a9b5750916001959492918387959310614a82575b505050811b018155614865565b5f1960f88560031b161c199101351690555f8080614a75565b91936020600181928787013581550195019201614a5c565b6040517f87dfb26700000000000000000000000000000000000000000000000000000000815260206004820152806119fe6024820187611f24565b906040519073ffffffffffffffffffffffffffffffffffffffff835192602081818701958087835e81017fc5779f3c2c21083eefa6d04f6a698bc0d8c10db124ad5e0df6ef394b6d7bf6008152030190205416614c165791614c0b9173ffffffffffffffffffffffffffffffffffffffff7fa6ec8e860960e638347460dc632fbe0175c51a5ca130e336138bbe26ff3044999416906020604051809285518091835e81017fc5779f3c2c21083eefa6d04f6a698bc0d8c10db124ad5e0df6ef394b6d7bf60081520301902073ffffffffffffffffffffffffffffffffffffffff82167fffffffffffffffffffffffff0000000000000000000000000000000000000000825416179055604051928392604084526040840190611f24565b9060208301520390a1565b6040517f837f46a600000000000000000000000000000000000000000000000000000000815260206004820152806119fe6024820186611f24565b600961205a916020937fffffffffffffffff000000000000000000000000000000000000000000000000856040519687948051918291018387015e840101917f0100000000000000000000000000000000000000000000000000000000000000835260c01b1660018201520301601f198101835282611fca565b602091604051614cdb8482611fca565b5f8152915f9160808201935b84518051851015614e335784614cfc91612342565b51865f818351604051918183925191829101835e8101838152039060025afa15610d5d575f5190875f8180840151604051918183925191829101835e8101838152039060025afa15610d5d575f5191885f816040850151604051918183925191829101835e8101838152039060025afa15610d5d575f5192895f816060860151604051918183925191829101835e8101838152039060025afa15610d5d57895f8160808251960151604051918183925191829101835e8101838152039060025afa15610d5d5789935f938451916040519387850195865260408501526060840152608083015260a082015260a08152614df660c082611fca565b604051918291518091835e8101838152039060025afa15610d5d57600190614e2b5f51916142916040519384928b840161420c565b930192614ce7565b509250939250825f816040840151604051918183925191829101835e8101838152039060025afa15610d5d57825f606081519301516040517fffffffffffffffff0000000000000000000000000000000000000000000000008482019260c01b16825260088152614ea5602882611fca565b604051918291518091835e8101838152039060025afa15610d5d57825f81815194604051918183925191829101835e8101838152039060025afa15610d5d575f91825160405191858301937f0200000000000000000000000000000000000000000000000000000000000000855260218401526041830152606182015260618152614300608182611fca565b614f61614f52613ad0614f4760408501856121fb565b919061284e866122ce565b60208151910120913690612f1b565b604051614f7681614291602082019485612e58565b51902090805f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e074760060205260405f20548281146144e25780614fe357505f527f1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e074760060205260405f2055600190565b90507f657b94fe000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b602073ffffffffffffffffffffffffffffffffffffffff7f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad9216807fffffffffffffffffffffffff00000000000000000000000000000000000000007ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055604051908152a1565b905f8091602081519101845af48080615172575b156150f25750506040513d81523d5f602083013e60203d82010160405290565b156151395773ffffffffffffffffffffffffffffffffffffffff907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d1561514a576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d1515806150d25750813b15156150d2565b600961205a916020937fffffffffffffffff000000000000000000000000000000000000000000000000856040519687948051918291018387015e840101917f0200000000000000000000000000000000000000000000000000000000000000835260c01b1660018201520301601f198101835282611fca565b805190825180831061526e5782808210911802808318921415820282189060206152298385612e4b565b928061524d61523786611fed565b956152456040519788611fca565b808752611fed565b95601f19848701970136883703920101835e51902090602081519101201490565b505050505f90565b805182118015615332575b6152db5760018211806152e3575b158015908160011b9182046002141715611a385760280180602811611a385782036152db5773ffffffffffffffffffffffffffffffffffffffff92915f6152d592615390565b90921690565b50505f905f90565b507f30780000000000000000000000000000000000000000000000000000000000007fffff0000000000000000000000000000000000000000000000000000000000006020830151161461528f565b505f615281565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c161561536857565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b92909260018401808511611a3857831180615446575b15938415948560011b9586046002141715611a38575f948101809111611a38579192905b8183106153da5750505060019190565b9092919360ff6154117fff000000000000000000000000000000000000000000000000000000000000006020888601015116615497565b16600f811161543b578160041b9180830460101490151715611a38576001910194019192906153ca565b505f94508493505050565b507f30780000000000000000000000000000000000000000000000000000000000007fffff0000000000000000000000000000000000000000000000000000000000006020868401015116146153a6565b60f81c602f811180615559575b156154d1577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd00160ff1690565b606081118061554f575b15615508577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa90160ff1690565b6040811180615545575b1561553f577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc90160ff1690565b5060ff90565b5060478110615512565b50606781106154db565b50603a81106154a456fea164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
}

// ContractABI is the input ABI used to generate the binding from.
// Deprecated: Use ContractMetaData.ABI instead.
var ContractABI = ContractMetaData.ABI

// ContractBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ContractMetaData.Bin instead.
var ContractBin = ContractMetaData.Bin

// DeployContract deploys a new Ethereum contract, binding an instance of Contract to it.
func DeployContract(auth *bind.TransactOpts, backend bind.ContractBackend) (common.Address, *types.Transaction, *Contract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ContractBin), backend)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &Contract{ContractCaller: ContractCaller{contract: contract}, ContractTransactor: ContractTransactor{contract: contract}, ContractFilterer: ContractFilterer{contract: contract}}, nil
}

// Contract is an auto generated Go binding around an Ethereum contract.
type Contract struct {
	ContractCaller     // Read-only binding to the contract
	ContractTransactor // Write-only binding to the contract
	ContractFilterer   // Log filterer for contract events
}

// ContractCaller is an auto generated read-only Go binding around an Ethereum contract.
type ContractCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ContractTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ContractFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ContractSession struct {
	Contract     *Contract         // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ContractCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ContractCallerSession struct {
	Contract *ContractCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts   // Call options to use throughout this session
}

// ContractTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ContractTransactorSession struct {
	Contract     *ContractTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// ContractRaw is an auto generated low-level Go binding around an Ethereum contract.
type ContractRaw struct {
	Contract *Contract // Generic contract binding to access the raw methods on
}

// ContractCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ContractCallerRaw struct {
	Contract *ContractCaller // Generic read-only contract binding to access the raw methods on
}

// ContractTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ContractTransactorRaw struct {
	Contract *ContractTransactor // Generic write-only contract binding to access the raw methods on
}

// NewContract creates a new instance of Contract, bound to a specific deployed contract.
func NewContract(address common.Address, backend bind.ContractBackend) (*Contract, error) {
	contract, err := bindContract(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &Contract{ContractCaller: ContractCaller{contract: contract}, ContractTransactor: ContractTransactor{contract: contract}, ContractFilterer: ContractFilterer{contract: contract}}, nil
}

// NewContractCaller creates a new read-only instance of Contract, bound to a specific deployed contract.
func NewContractCaller(address common.Address, caller bind.ContractCaller) (*ContractCaller, error) {
	contract, err := bindContract(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ContractCaller{contract: contract}, nil
}

// NewContractTransactor creates a new write-only instance of Contract, bound to a specific deployed contract.
func NewContractTransactor(address common.Address, transactor bind.ContractTransactor) (*ContractTransactor, error) {
	contract, err := bindContract(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ContractTransactor{contract: contract}, nil
}

// NewContractFilterer creates a new log filterer instance of Contract, bound to a specific deployed contract.
func NewContractFilterer(address common.Address, filterer bind.ContractFilterer) (*ContractFilterer, error) {
	contract, err := bindContract(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ContractFilterer{contract: contract}, nil
}

// bindContract binds a generic wrapper to an already deployed contract.
func bindContract(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Contract *ContractRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Contract.Contract.ContractCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Contract *ContractRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.Contract.ContractTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Contract *ContractRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Contract.Contract.ContractTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Contract *ContractCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Contract.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Contract *ContractTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Contract *ContractTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Contract.Contract.contract.Transact(opts, method, params...)
}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_Contract *ContractCaller) UPGRADEINTERFACEVERSION(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "UPGRADE_INTERFACE_VERSION")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_Contract *ContractSession) UPGRADEINTERFACEVERSION() (string, error) {
	return _Contract.Contract.UPGRADEINTERFACEVERSION(&_Contract.CallOpts)
}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_Contract *ContractCallerSession) UPGRADEINTERFACEVERSION() (string, error) {
	return _Contract.Contract.UPGRADEINTERFACEVERSION(&_Contract.CallOpts)
}

// Authority is a free data retrieval call binding the contract method 0xbf7e214f.
//
// Solidity: function authority() view returns(address)
func (_Contract *ContractCaller) Authority(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "authority")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Authority is a free data retrieval call binding the contract method 0xbf7e214f.
//
// Solidity: function authority() view returns(address)
func (_Contract *ContractSession) Authority() (common.Address, error) {
	return _Contract.Contract.Authority(&_Contract.CallOpts)
}

// Authority is a free data retrieval call binding the contract method 0xbf7e214f.
//
// Solidity: function authority() view returns(address)
func (_Contract *ContractCallerSession) Authority() (common.Address, error) {
	return _Contract.Contract.Authority(&_Contract.CallOpts)
}

// GetClient is a free data retrieval call binding the contract method 0x7eb78932.
//
// Solidity: function getClient(string clientId) view returns(address)
func (_Contract *ContractCaller) GetClient(opts *bind.CallOpts, clientId string) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getClient", clientId)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetClient is a free data retrieval call binding the contract method 0x7eb78932.
//
// Solidity: function getClient(string clientId) view returns(address)
func (_Contract *ContractSession) GetClient(clientId string) (common.Address, error) {
	return _Contract.Contract.GetClient(&_Contract.CallOpts, clientId)
}

// GetClient is a free data retrieval call binding the contract method 0x7eb78932.
//
// Solidity: function getClient(string clientId) view returns(address)
func (_Contract *ContractCallerSession) GetClient(clientId string) (common.Address, error) {
	return _Contract.Contract.GetClient(&_Contract.CallOpts, clientId)
}

// GetCommitment is a free data retrieval call binding the contract method 0x7795820c.
//
// Solidity: function getCommitment(bytes32 hashedPath) view returns(bytes32)
func (_Contract *ContractCaller) GetCommitment(opts *bind.CallOpts, hashedPath [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getCommitment", hashedPath)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetCommitment is a free data retrieval call binding the contract method 0x7795820c.
//
// Solidity: function getCommitment(bytes32 hashedPath) view returns(bytes32)
func (_Contract *ContractSession) GetCommitment(hashedPath [32]byte) ([32]byte, error) {
	return _Contract.Contract.GetCommitment(&_Contract.CallOpts, hashedPath)
}

// GetCommitment is a free data retrieval call binding the contract method 0x7795820c.
//
// Solidity: function getCommitment(bytes32 hashedPath) view returns(bytes32)
func (_Contract *ContractCallerSession) GetCommitment(hashedPath [32]byte) ([32]byte, error) {
	return _Contract.Contract.GetCommitment(&_Contract.CallOpts, hashedPath)
}

// GetCounterparty is a free data retrieval call binding the contract method 0xb0777bfa.
//
// Solidity: function getCounterparty(string clientId) view returns((string,bytes[]))
func (_Contract *ContractCaller) GetCounterparty(opts *bind.CallOpts, clientId string) (IICS02ClientMsgsCounterpartyInfo, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getCounterparty", clientId)

	if err != nil {
		return *new(IICS02ClientMsgsCounterpartyInfo), err
	}

	out0 := *abi.ConvertType(out[0], new(IICS02ClientMsgsCounterpartyInfo)).(*IICS02ClientMsgsCounterpartyInfo)

	return out0, err

}

// GetCounterparty is a free data retrieval call binding the contract method 0xb0777bfa.
//
// Solidity: function getCounterparty(string clientId) view returns((string,bytes[]))
func (_Contract *ContractSession) GetCounterparty(clientId string) (IICS02ClientMsgsCounterpartyInfo, error) {
	return _Contract.Contract.GetCounterparty(&_Contract.CallOpts, clientId)
}

// GetCounterparty is a free data retrieval call binding the contract method 0xb0777bfa.
//
// Solidity: function getCounterparty(string clientId) view returns((string,bytes[]))
func (_Contract *ContractCallerSession) GetCounterparty(clientId string) (IICS02ClientMsgsCounterpartyInfo, error) {
	return _Contract.Contract.GetCounterparty(&_Contract.CallOpts, clientId)
}

// GetIBCApp is a free data retrieval call binding the contract method 0x2447af29.
//
// Solidity: function getIBCApp(string portId) view returns(address)
func (_Contract *ContractCaller) GetIBCApp(opts *bind.CallOpts, portId string) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIBCApp", portId)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetIBCApp is a free data retrieval call binding the contract method 0x2447af29.
//
// Solidity: function getIBCApp(string portId) view returns(address)
func (_Contract *ContractSession) GetIBCApp(portId string) (common.Address, error) {
	return _Contract.Contract.GetIBCApp(&_Contract.CallOpts, portId)
}

// GetIBCApp is a free data retrieval call binding the contract method 0x2447af29.
//
// Solidity: function getIBCApp(string portId) view returns(address)
func (_Contract *ContractCallerSession) GetIBCApp(portId string) (common.Address, error) {
	return _Contract.Contract.GetIBCApp(&_Contract.CallOpts, portId)
}

// GetNextClientSeq is a free data retrieval call binding the contract method 0x27f146f3.
//
// Solidity: function getNextClientSeq() view returns(uint256)
func (_Contract *ContractCaller) GetNextClientSeq(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getNextClientSeq")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetNextClientSeq is a free data retrieval call binding the contract method 0x27f146f3.
//
// Solidity: function getNextClientSeq() view returns(uint256)
func (_Contract *ContractSession) GetNextClientSeq() (*big.Int, error) {
	return _Contract.Contract.GetNextClientSeq(&_Contract.CallOpts)
}

// GetNextClientSeq is a free data retrieval call binding the contract method 0x27f146f3.
//
// Solidity: function getNextClientSeq() view returns(uint256)
func (_Contract *ContractCallerSession) GetNextClientSeq() (*big.Int, error) {
	return _Contract.Contract.GetNextClientSeq(&_Contract.CallOpts)
}

// IsConsumingScheduledOp is a free data retrieval call binding the contract method 0x8fb36037.
//
// Solidity: function isConsumingScheduledOp() view returns(bytes4)
func (_Contract *ContractCaller) IsConsumingScheduledOp(opts *bind.CallOpts) ([4]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "isConsumingScheduledOp")

	if err != nil {
		return *new([4]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([4]byte)).(*[4]byte)

	return out0, err

}

// IsConsumingScheduledOp is a free data retrieval call binding the contract method 0x8fb36037.
//
// Solidity: function isConsumingScheduledOp() view returns(bytes4)
func (_Contract *ContractSession) IsConsumingScheduledOp() ([4]byte, error) {
	return _Contract.Contract.IsConsumingScheduledOp(&_Contract.CallOpts)
}

// IsConsumingScheduledOp is a free data retrieval call binding the contract method 0x8fb36037.
//
// Solidity: function isConsumingScheduledOp() view returns(bytes4)
func (_Contract *ContractCallerSession) IsConsumingScheduledOp() ([4]byte, error) {
	return _Contract.Contract.IsConsumingScheduledOp(&_Contract.CallOpts)
}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_Contract *ContractCaller) ProxiableUUID(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "proxiableUUID")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_Contract *ContractSession) ProxiableUUID() ([32]byte, error) {
	return _Contract.Contract.ProxiableUUID(&_Contract.CallOpts)
}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_Contract *ContractCallerSession) ProxiableUUID() ([32]byte, error) {
	return _Contract.Contract.ProxiableUUID(&_Contract.CallOpts)
}

// AckPacket is a paid mutator transaction binding the contract method 0x1bca011a.
//
// Solidity: function ackPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractTransactor) AckPacket(opts *bind.TransactOpts, msg_ IICS26RouterMsgsMsgAckPacket) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "ackPacket", msg_)
}

// AckPacket is a paid mutator transaction binding the contract method 0x1bca011a.
//
// Solidity: function ackPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractSession) AckPacket(msg_ IICS26RouterMsgsMsgAckPacket) (*types.Transaction, error) {
	return _Contract.Contract.AckPacket(&_Contract.TransactOpts, msg_)
}

// AckPacket is a paid mutator transaction binding the contract method 0x1bca011a.
//
// Solidity: function ackPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractTransactorSession) AckPacket(msg_ IICS26RouterMsgsMsgAckPacket) (*types.Transaction, error) {
	return _Contract.Contract.AckPacket(&_Contract.TransactOpts, msg_)
}

// AddClient is a paid mutator transaction binding the contract method 0x1ec43e23.
//
// Solidity: function addClient(string clientId, (string,bytes[]) counterpartyInfo, address client) returns(string)
func (_Contract *ContractTransactor) AddClient(opts *bind.TransactOpts, clientId string, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "addClient", clientId, counterpartyInfo, client)
}

// AddClient is a paid mutator transaction binding the contract method 0x1ec43e23.
//
// Solidity: function addClient(string clientId, (string,bytes[]) counterpartyInfo, address client) returns(string)
func (_Contract *ContractSession) AddClient(clientId string, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddClient(&_Contract.TransactOpts, clientId, counterpartyInfo, client)
}

// AddClient is a paid mutator transaction binding the contract method 0x1ec43e23.
//
// Solidity: function addClient(string clientId, (string,bytes[]) counterpartyInfo, address client) returns(string)
func (_Contract *ContractTransactorSession) AddClient(clientId string, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddClient(&_Contract.TransactOpts, clientId, counterpartyInfo, client)
}

// AddClient0 is a paid mutator transaction binding the contract method 0xe3cb36a0.
//
// Solidity: function addClient((string,bytes[]) counterpartyInfo, address client) returns(string)
func (_Contract *ContractTransactor) AddClient0(opts *bind.TransactOpts, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "addClient0", counterpartyInfo, client)
}

// AddClient0 is a paid mutator transaction binding the contract method 0xe3cb36a0.
//
// Solidity: function addClient((string,bytes[]) counterpartyInfo, address client) returns(string)
func (_Contract *ContractSession) AddClient0(counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddClient0(&_Contract.TransactOpts, counterpartyInfo, client)
}

// AddClient0 is a paid mutator transaction binding the contract method 0xe3cb36a0.
//
// Solidity: function addClient((string,bytes[]) counterpartyInfo, address client) returns(string)
func (_Contract *ContractTransactorSession) AddClient0(counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddClient0(&_Contract.TransactOpts, counterpartyInfo, client)
}

// AddIBCApp is a paid mutator transaction binding the contract method 0x4b720d5b.
//
// Solidity: function addIBCApp(address app) returns()
func (_Contract *ContractTransactor) AddIBCApp(opts *bind.TransactOpts, app common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "addIBCApp", app)
}

// AddIBCApp is a paid mutator transaction binding the contract method 0x4b720d5b.
//
// Solidity: function addIBCApp(address app) returns()
func (_Contract *ContractSession) AddIBCApp(app common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddIBCApp(&_Contract.TransactOpts, app)
}

// AddIBCApp is a paid mutator transaction binding the contract method 0x4b720d5b.
//
// Solidity: function addIBCApp(address app) returns()
func (_Contract *ContractTransactorSession) AddIBCApp(app common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddIBCApp(&_Contract.TransactOpts, app)
}

// AddIBCApp0 is a paid mutator transaction binding the contract method 0x5f516889.
//
// Solidity: function addIBCApp(string portId, address app) returns()
func (_Contract *ContractTransactor) AddIBCApp0(opts *bind.TransactOpts, portId string, app common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "addIBCApp0", portId, app)
}

// AddIBCApp0 is a paid mutator transaction binding the contract method 0x5f516889.
//
// Solidity: function addIBCApp(string portId, address app) returns()
func (_Contract *ContractSession) AddIBCApp0(portId string, app common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddIBCApp0(&_Contract.TransactOpts, portId, app)
}

// AddIBCApp0 is a paid mutator transaction binding the contract method 0x5f516889.
//
// Solidity: function addIBCApp(string portId, address app) returns()
func (_Contract *ContractTransactorSession) AddIBCApp0(portId string, app common.Address) (*types.Transaction, error) {
	return _Contract.Contract.AddIBCApp0(&_Contract.TransactOpts, portId, app)
}

// Initialize is a paid mutator transaction binding the contract method 0xc4d66de8.
//
// Solidity: function initialize(address authority) returns()
func (_Contract *ContractTransactor) Initialize(opts *bind.TransactOpts, authority common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "initialize", authority)
}

// Initialize is a paid mutator transaction binding the contract method 0xc4d66de8.
//
// Solidity: function initialize(address authority) returns()
func (_Contract *ContractSession) Initialize(authority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, authority)
}

// Initialize is a paid mutator transaction binding the contract method 0xc4d66de8.
//
// Solidity: function initialize(address authority) returns()
func (_Contract *ContractTransactorSession) Initialize(authority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, authority)
}

// MigrateClient is a paid mutator transaction binding the contract method 0xcce0b265.
//
// Solidity: function migrateClient(string clientId, (string,bytes[]) counterpartyInfo, address client) returns()
func (_Contract *ContractTransactor) MigrateClient(opts *bind.TransactOpts, clientId string, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "migrateClient", clientId, counterpartyInfo, client)
}

// MigrateClient is a paid mutator transaction binding the contract method 0xcce0b265.
//
// Solidity: function migrateClient(string clientId, (string,bytes[]) counterpartyInfo, address client) returns()
func (_Contract *ContractSession) MigrateClient(clientId string, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.Contract.MigrateClient(&_Contract.TransactOpts, clientId, counterpartyInfo, client)
}

// MigrateClient is a paid mutator transaction binding the contract method 0xcce0b265.
//
// Solidity: function migrateClient(string clientId, (string,bytes[]) counterpartyInfo, address client) returns()
func (_Contract *ContractTransactorSession) MigrateClient(clientId string, counterpartyInfo IICS02ClientMsgsCounterpartyInfo, client common.Address) (*types.Transaction, error) {
	return _Contract.Contract.MigrateClient(&_Contract.TransactOpts, clientId, counterpartyInfo, client)
}

// Multicall is a paid mutator transaction binding the contract method 0xac9650d8.
//
// Solidity: function multicall(bytes[] data) returns(bytes[] results)
func (_Contract *ContractTransactor) Multicall(opts *bind.TransactOpts, data [][]byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "multicall", data)
}

// Multicall is a paid mutator transaction binding the contract method 0xac9650d8.
//
// Solidity: function multicall(bytes[] data) returns(bytes[] results)
func (_Contract *ContractSession) Multicall(data [][]byte) (*types.Transaction, error) {
	return _Contract.Contract.Multicall(&_Contract.TransactOpts, data)
}

// Multicall is a paid mutator transaction binding the contract method 0xac9650d8.
//
// Solidity: function multicall(bytes[] data) returns(bytes[] results)
func (_Contract *ContractTransactorSession) Multicall(data [][]byte) (*types.Transaction, error) {
	return _Contract.Contract.Multicall(&_Contract.TransactOpts, data)
}

// RecvPacket is a paid mutator transaction binding the contract method 0x5ebd10ca.
//
// Solidity: function recvPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractTransactor) RecvPacket(opts *bind.TransactOpts, msg_ IICS26RouterMsgsMsgRecvPacket) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "recvPacket", msg_)
}

// RecvPacket is a paid mutator transaction binding the contract method 0x5ebd10ca.
//
// Solidity: function recvPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractSession) RecvPacket(msg_ IICS26RouterMsgsMsgRecvPacket) (*types.Transaction, error) {
	return _Contract.Contract.RecvPacket(&_Contract.TransactOpts, msg_)
}

// RecvPacket is a paid mutator transaction binding the contract method 0x5ebd10ca.
//
// Solidity: function recvPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractTransactorSession) RecvPacket(msg_ IICS26RouterMsgsMsgRecvPacket) (*types.Transaction, error) {
	return _Contract.Contract.RecvPacket(&_Contract.TransactOpts, msg_)
}

// SendPacket is a paid mutator transaction binding the contract method 0x4d6e7ce3.
//
// Solidity: function sendPacket((string,uint64,(string,string,string,string,bytes)) msg_) returns(uint64)
func (_Contract *ContractTransactor) SendPacket(opts *bind.TransactOpts, msg_ IICS26RouterMsgsMsgSendPacket) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "sendPacket", msg_)
}

// SendPacket is a paid mutator transaction binding the contract method 0x4d6e7ce3.
//
// Solidity: function sendPacket((string,uint64,(string,string,string,string,bytes)) msg_) returns(uint64)
func (_Contract *ContractSession) SendPacket(msg_ IICS26RouterMsgsMsgSendPacket) (*types.Transaction, error) {
	return _Contract.Contract.SendPacket(&_Contract.TransactOpts, msg_)
}

// SendPacket is a paid mutator transaction binding the contract method 0x4d6e7ce3.
//
// Solidity: function sendPacket((string,uint64,(string,string,string,string,bytes)) msg_) returns(uint64)
func (_Contract *ContractTransactorSession) SendPacket(msg_ IICS26RouterMsgsMsgSendPacket) (*types.Transaction, error) {
	return _Contract.Contract.SendPacket(&_Contract.TransactOpts, msg_)
}

// SetAuthority is a paid mutator transaction binding the contract method 0x7a9e5e4b.
//
// Solidity: function setAuthority(address newAuthority) returns()
func (_Contract *ContractTransactor) SetAuthority(opts *bind.TransactOpts, newAuthority common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "setAuthority", newAuthority)
}

// SetAuthority is a paid mutator transaction binding the contract method 0x7a9e5e4b.
//
// Solidity: function setAuthority(address newAuthority) returns()
func (_Contract *ContractSession) SetAuthority(newAuthority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SetAuthority(&_Contract.TransactOpts, newAuthority)
}

// SetAuthority is a paid mutator transaction binding the contract method 0x7a9e5e4b.
//
// Solidity: function setAuthority(address newAuthority) returns()
func (_Contract *ContractTransactorSession) SetAuthority(newAuthority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SetAuthority(&_Contract.TransactOpts, newAuthority)
}

// SubmitMisbehaviour is a paid mutator transaction binding the contract method 0x9e2e5c83.
//
// Solidity: function submitMisbehaviour(string clientId, bytes misbehaviourMsg) returns()
func (_Contract *ContractTransactor) SubmitMisbehaviour(opts *bind.TransactOpts, clientId string, misbehaviourMsg []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "submitMisbehaviour", clientId, misbehaviourMsg)
}

// SubmitMisbehaviour is a paid mutator transaction binding the contract method 0x9e2e5c83.
//
// Solidity: function submitMisbehaviour(string clientId, bytes misbehaviourMsg) returns()
func (_Contract *ContractSession) SubmitMisbehaviour(clientId string, misbehaviourMsg []byte) (*types.Transaction, error) {
	return _Contract.Contract.SubmitMisbehaviour(&_Contract.TransactOpts, clientId, misbehaviourMsg)
}

// SubmitMisbehaviour is a paid mutator transaction binding the contract method 0x9e2e5c83.
//
// Solidity: function submitMisbehaviour(string clientId, bytes misbehaviourMsg) returns()
func (_Contract *ContractTransactorSession) SubmitMisbehaviour(clientId string, misbehaviourMsg []byte) (*types.Transaction, error) {
	return _Contract.Contract.SubmitMisbehaviour(&_Contract.TransactOpts, clientId, misbehaviourMsg)
}

// TimeoutPacket is a paid mutator transaction binding the contract method 0xb98c330a.
//
// Solidity: function timeoutPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractTransactor) TimeoutPacket(opts *bind.TransactOpts, msg_ IICS26RouterMsgsMsgTimeoutPacket) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "timeoutPacket", msg_)
}

// TimeoutPacket is a paid mutator transaction binding the contract method 0xb98c330a.
//
// Solidity: function timeoutPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractSession) TimeoutPacket(msg_ IICS26RouterMsgsMsgTimeoutPacket) (*types.Transaction, error) {
	return _Contract.Contract.TimeoutPacket(&_Contract.TransactOpts, msg_)
}

// TimeoutPacket is a paid mutator transaction binding the contract method 0xb98c330a.
//
// Solidity: function timeoutPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)) msg_) returns()
func (_Contract *ContractTransactorSession) TimeoutPacket(msg_ IICS26RouterMsgsMsgTimeoutPacket) (*types.Transaction, error) {
	return _Contract.Contract.TimeoutPacket(&_Contract.TransactOpts, msg_)
}

// UpdateClient is a paid mutator transaction binding the contract method 0x6fbf8079.
//
// Solidity: function updateClient(string clientId, bytes updateMsg) returns(uint8)
func (_Contract *ContractTransactor) UpdateClient(opts *bind.TransactOpts, clientId string, updateMsg []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "updateClient", clientId, updateMsg)
}

// UpdateClient is a paid mutator transaction binding the contract method 0x6fbf8079.
//
// Solidity: function updateClient(string clientId, bytes updateMsg) returns(uint8)
func (_Contract *ContractSession) UpdateClient(clientId string, updateMsg []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpdateClient(&_Contract.TransactOpts, clientId, updateMsg)
}

// UpdateClient is a paid mutator transaction binding the contract method 0x6fbf8079.
//
// Solidity: function updateClient(string clientId, bytes updateMsg) returns(uint8)
func (_Contract *ContractTransactorSession) UpdateClient(clientId string, updateMsg []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpdateClient(&_Contract.TransactOpts, clientId, updateMsg)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_Contract *ContractTransactor) UpgradeToAndCall(opts *bind.TransactOpts, newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "upgradeToAndCall", newImplementation, data)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_Contract *ContractSession) UpgradeToAndCall(newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeToAndCall(&_Contract.TransactOpts, newImplementation, data)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_Contract *ContractTransactorSession) UpgradeToAndCall(newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeToAndCall(&_Contract.TransactOpts, newImplementation, data)
}

// VerifyMembership is a paid mutator transaction binding the contract method 0x55acb619.
//
// Solidity: function verifyMembership(string clientId, (bytes,(uint64,uint64),bytes[],bytes) msg_) returns(uint256)
func (_Contract *ContractTransactor) VerifyMembership(opts *bind.TransactOpts, clientId string, msg_ ILightClientMsgsMsgVerifyMembership) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "verifyMembership", clientId, msg_)
}

// VerifyMembership is a paid mutator transaction binding the contract method 0x55acb619.
//
// Solidity: function verifyMembership(string clientId, (bytes,(uint64,uint64),bytes[],bytes) msg_) returns(uint256)
func (_Contract *ContractSession) VerifyMembership(clientId string, msg_ ILightClientMsgsMsgVerifyMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyMembership(&_Contract.TransactOpts, clientId, msg_)
}

// VerifyMembership is a paid mutator transaction binding the contract method 0x55acb619.
//
// Solidity: function verifyMembership(string clientId, (bytes,(uint64,uint64),bytes[],bytes) msg_) returns(uint256)
func (_Contract *ContractTransactorSession) VerifyMembership(clientId string, msg_ ILightClientMsgsMsgVerifyMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyMembership(&_Contract.TransactOpts, clientId, msg_)
}

// VerifyNonMembership is a paid mutator transaction binding the contract method 0x4481f4d2.
//
// Solidity: function verifyNonMembership(string clientId, (bytes,(uint64,uint64),bytes[]) msg_) returns(uint256)
func (_Contract *ContractTransactor) VerifyNonMembership(opts *bind.TransactOpts, clientId string, msg_ ILightClientMsgsMsgVerifyNonMembership) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "verifyNonMembership", clientId, msg_)
}

// VerifyNonMembership is a paid mutator transaction binding the contract method 0x4481f4d2.
//
// Solidity: function verifyNonMembership(string clientId, (bytes,(uint64,uint64),bytes[]) msg_) returns(uint256)
func (_Contract *ContractSession) VerifyNonMembership(clientId string, msg_ ILightClientMsgsMsgVerifyNonMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyNonMembership(&_Contract.TransactOpts, clientId, msg_)
}

// VerifyNonMembership is a paid mutator transaction binding the contract method 0x4481f4d2.
//
// Solidity: function verifyNonMembership(string clientId, (bytes,(uint64,uint64),bytes[]) msg_) returns(uint256)
func (_Contract *ContractTransactorSession) VerifyNonMembership(clientId string, msg_ ILightClientMsgsMsgVerifyNonMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyNonMembership(&_Contract.TransactOpts, clientId, msg_)
}

// ContractAckPacketIterator is returned from FilterAckPacket and is used to iterate over the raw logs and unpacked data for AckPacket events raised by the Contract contract.
type ContractAckPacketIterator struct {
	Event *ContractAckPacket // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractAckPacketIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractAckPacket)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractAckPacket)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractAckPacketIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractAckPacketIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractAckPacket represents a AckPacket event raised by the Contract contract.
type ContractAckPacket struct {
	ClientId        common.Hash
	Sequence        *big.Int
	Packet          IICS26RouterMsgsPacket
	Acknowledgement []byte
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterAckPacket is a free log retrieval operation binding the contract event 0xf9bab74bcdb634f4d3dd064cc42a13df056598e1c0336905d2f5750fbfb08b7b.
//
// Solidity: event AckPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet, bytes acknowledgement)
func (_Contract *ContractFilterer) FilterAckPacket(opts *bind.FilterOpts, clientId []string, sequence []*big.Int) (*ContractAckPacketIterator, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "AckPacket", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return &ContractAckPacketIterator{contract: _Contract.contract, event: "AckPacket", logs: logs, sub: sub}, nil
}

// WatchAckPacket is a free log subscription operation binding the contract event 0xf9bab74bcdb634f4d3dd064cc42a13df056598e1c0336905d2f5750fbfb08b7b.
//
// Solidity: event AckPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet, bytes acknowledgement)
func (_Contract *ContractFilterer) WatchAckPacket(opts *bind.WatchOpts, sink chan<- *ContractAckPacket, clientId []string, sequence []*big.Int) (event.Subscription, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "AckPacket", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractAckPacket)
				if err := _Contract.contract.UnpackLog(event, "AckPacket", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseAckPacket is a log parse operation binding the contract event 0xf9bab74bcdb634f4d3dd064cc42a13df056598e1c0336905d2f5750fbfb08b7b.
//
// Solidity: event AckPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet, bytes acknowledgement)
func (_Contract *ContractFilterer) ParseAckPacket(log types.Log) (*ContractAckPacket, error) {
	event := new(ContractAckPacket)
	if err := _Contract.contract.UnpackLog(event, "AckPacket", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractAuthorityUpdatedIterator is returned from FilterAuthorityUpdated and is used to iterate over the raw logs and unpacked data for AuthorityUpdated events raised by the Contract contract.
type ContractAuthorityUpdatedIterator struct {
	Event *ContractAuthorityUpdated // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractAuthorityUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractAuthorityUpdated)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractAuthorityUpdated)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractAuthorityUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractAuthorityUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractAuthorityUpdated represents a AuthorityUpdated event raised by the Contract contract.
type ContractAuthorityUpdated struct {
	Authority common.Address
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterAuthorityUpdated is a free log retrieval operation binding the contract event 0x2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad.
//
// Solidity: event AuthorityUpdated(address authority)
func (_Contract *ContractFilterer) FilterAuthorityUpdated(opts *bind.FilterOpts) (*ContractAuthorityUpdatedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "AuthorityUpdated")
	if err != nil {
		return nil, err
	}
	return &ContractAuthorityUpdatedIterator{contract: _Contract.contract, event: "AuthorityUpdated", logs: logs, sub: sub}, nil
}

// WatchAuthorityUpdated is a free log subscription operation binding the contract event 0x2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad.
//
// Solidity: event AuthorityUpdated(address authority)
func (_Contract *ContractFilterer) WatchAuthorityUpdated(opts *bind.WatchOpts, sink chan<- *ContractAuthorityUpdated) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "AuthorityUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractAuthorityUpdated)
				if err := _Contract.contract.UnpackLog(event, "AuthorityUpdated", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseAuthorityUpdated is a log parse operation binding the contract event 0x2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad.
//
// Solidity: event AuthorityUpdated(address authority)
func (_Contract *ContractFilterer) ParseAuthorityUpdated(log types.Log) (*ContractAuthorityUpdated, error) {
	event := new(ContractAuthorityUpdated)
	if err := _Contract.contract.UnpackLog(event, "AuthorityUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIBCAppAddedIterator is returned from FilterIBCAppAdded and is used to iterate over the raw logs and unpacked data for IBCAppAdded events raised by the Contract contract.
type ContractIBCAppAddedIterator struct {
	Event *ContractIBCAppAdded // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIBCAppAddedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIBCAppAdded)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIBCAppAdded)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIBCAppAddedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIBCAppAddedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIBCAppAdded represents a IBCAppAdded event raised by the Contract contract.
type ContractIBCAppAdded struct {
	PortId string
	App    common.Address
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterIBCAppAdded is a free log retrieval operation binding the contract event 0xa6ec8e860960e638347460dc632fbe0175c51a5ca130e336138bbe26ff304499.
//
// Solidity: event IBCAppAdded(string portId, address app)
func (_Contract *ContractFilterer) FilterIBCAppAdded(opts *bind.FilterOpts) (*ContractIBCAppAddedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IBCAppAdded")
	if err != nil {
		return nil, err
	}
	return &ContractIBCAppAddedIterator{contract: _Contract.contract, event: "IBCAppAdded", logs: logs, sub: sub}, nil
}

// WatchIBCAppAdded is a free log subscription operation binding the contract event 0xa6ec8e860960e638347460dc632fbe0175c51a5ca130e336138bbe26ff304499.
//
// Solidity: event IBCAppAdded(string portId, address app)
func (_Contract *ContractFilterer) WatchIBCAppAdded(opts *bind.WatchOpts, sink chan<- *ContractIBCAppAdded) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IBCAppAdded")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIBCAppAdded)
				if err := _Contract.contract.UnpackLog(event, "IBCAppAdded", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIBCAppAdded is a log parse operation binding the contract event 0xa6ec8e860960e638347460dc632fbe0175c51a5ca130e336138bbe26ff304499.
//
// Solidity: event IBCAppAdded(string portId, address app)
func (_Contract *ContractFilterer) ParseIBCAppAdded(log types.Log) (*ContractIBCAppAdded, error) {
	event := new(ContractIBCAppAdded)
	if err := _Contract.contract.UnpackLog(event, "IBCAppAdded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIBCAppRecvPacketCallbackErrorIterator is returned from FilterIBCAppRecvPacketCallbackError and is used to iterate over the raw logs and unpacked data for IBCAppRecvPacketCallbackError events raised by the Contract contract.
type ContractIBCAppRecvPacketCallbackErrorIterator struct {
	Event *ContractIBCAppRecvPacketCallbackError // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIBCAppRecvPacketCallbackErrorIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIBCAppRecvPacketCallbackError)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIBCAppRecvPacketCallbackError)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIBCAppRecvPacketCallbackErrorIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIBCAppRecvPacketCallbackErrorIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIBCAppRecvPacketCallbackError represents a IBCAppRecvPacketCallbackError event raised by the Contract contract.
type ContractIBCAppRecvPacketCallbackError struct {
	Reason []byte
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterIBCAppRecvPacketCallbackError is a free log retrieval operation binding the contract event 0xb9edb487876e8be10f54e377c1a815a54ad92a6db1c9561dfe8fad2f0d1da84f.
//
// Solidity: event IBCAppRecvPacketCallbackError(bytes reason)
func (_Contract *ContractFilterer) FilterIBCAppRecvPacketCallbackError(opts *bind.FilterOpts) (*ContractIBCAppRecvPacketCallbackErrorIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IBCAppRecvPacketCallbackError")
	if err != nil {
		return nil, err
	}
	return &ContractIBCAppRecvPacketCallbackErrorIterator{contract: _Contract.contract, event: "IBCAppRecvPacketCallbackError", logs: logs, sub: sub}, nil
}

// WatchIBCAppRecvPacketCallbackError is a free log subscription operation binding the contract event 0xb9edb487876e8be10f54e377c1a815a54ad92a6db1c9561dfe8fad2f0d1da84f.
//
// Solidity: event IBCAppRecvPacketCallbackError(bytes reason)
func (_Contract *ContractFilterer) WatchIBCAppRecvPacketCallbackError(opts *bind.WatchOpts, sink chan<- *ContractIBCAppRecvPacketCallbackError) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IBCAppRecvPacketCallbackError")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIBCAppRecvPacketCallbackError)
				if err := _Contract.contract.UnpackLog(event, "IBCAppRecvPacketCallbackError", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIBCAppRecvPacketCallbackError is a log parse operation binding the contract event 0xb9edb487876e8be10f54e377c1a815a54ad92a6db1c9561dfe8fad2f0d1da84f.
//
// Solidity: event IBCAppRecvPacketCallbackError(bytes reason)
func (_Contract *ContractFilterer) ParseIBCAppRecvPacketCallbackError(log types.Log) (*ContractIBCAppRecvPacketCallbackError, error) {
	event := new(ContractIBCAppRecvPacketCallbackError)
	if err := _Contract.contract.UnpackLog(event, "IBCAppRecvPacketCallbackError", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractICS02ClientAddedIterator is returned from FilterICS02ClientAdded and is used to iterate over the raw logs and unpacked data for ICS02ClientAdded events raised by the Contract contract.
type ContractICS02ClientAddedIterator struct {
	Event *ContractICS02ClientAdded // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractICS02ClientAddedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractICS02ClientAdded)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractICS02ClientAdded)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractICS02ClientAddedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractICS02ClientAddedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractICS02ClientAdded represents a ICS02ClientAdded event raised by the Contract contract.
type ContractICS02ClientAdded struct {
	ClientId         string
	CounterpartyInfo IICS02ClientMsgsCounterpartyInfo
	Client           common.Address
	Raw              types.Log // Blockchain specific contextual infos
}

// FilterICS02ClientAdded is a free log retrieval operation binding the contract event 0x0ecded31ecd211a73abf0fb3bc09150bbe321a05550fbe29ea0f16b6e25fbfa8.
//
// Solidity: event ICS02ClientAdded(string clientId, (string,bytes[]) counterpartyInfo, address client)
func (_Contract *ContractFilterer) FilterICS02ClientAdded(opts *bind.FilterOpts) (*ContractICS02ClientAddedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "ICS02ClientAdded")
	if err != nil {
		return nil, err
	}
	return &ContractICS02ClientAddedIterator{contract: _Contract.contract, event: "ICS02ClientAdded", logs: logs, sub: sub}, nil
}

// WatchICS02ClientAdded is a free log subscription operation binding the contract event 0x0ecded31ecd211a73abf0fb3bc09150bbe321a05550fbe29ea0f16b6e25fbfa8.
//
// Solidity: event ICS02ClientAdded(string clientId, (string,bytes[]) counterpartyInfo, address client)
func (_Contract *ContractFilterer) WatchICS02ClientAdded(opts *bind.WatchOpts, sink chan<- *ContractICS02ClientAdded) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "ICS02ClientAdded")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractICS02ClientAdded)
				if err := _Contract.contract.UnpackLog(event, "ICS02ClientAdded", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseICS02ClientAdded is a log parse operation binding the contract event 0x0ecded31ecd211a73abf0fb3bc09150bbe321a05550fbe29ea0f16b6e25fbfa8.
//
// Solidity: event ICS02ClientAdded(string clientId, (string,bytes[]) counterpartyInfo, address client)
func (_Contract *ContractFilterer) ParseICS02ClientAdded(log types.Log) (*ContractICS02ClientAdded, error) {
	event := new(ContractICS02ClientAdded)
	if err := _Contract.contract.UnpackLog(event, "ICS02ClientAdded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractICS02ClientMigratedIterator is returned from FilterICS02ClientMigrated and is used to iterate over the raw logs and unpacked data for ICS02ClientMigrated events raised by the Contract contract.
type ContractICS02ClientMigratedIterator struct {
	Event *ContractICS02ClientMigrated // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractICS02ClientMigratedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractICS02ClientMigrated)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractICS02ClientMigrated)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractICS02ClientMigratedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractICS02ClientMigratedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractICS02ClientMigrated represents a ICS02ClientMigrated event raised by the Contract contract.
type ContractICS02ClientMigrated struct {
	ClientId         string
	CounterpartyInfo IICS02ClientMsgsCounterpartyInfo
	Client           common.Address
	Raw              types.Log // Blockchain specific contextual infos
}

// FilterICS02ClientMigrated is a free log retrieval operation binding the contract event 0x23c2e29d6ae84e79fa116b8afd6e28ddc1de7f473d3edb407fbd08093c3ed6bf.
//
// Solidity: event ICS02ClientMigrated(string clientId, (string,bytes[]) counterpartyInfo, address client)
func (_Contract *ContractFilterer) FilterICS02ClientMigrated(opts *bind.FilterOpts) (*ContractICS02ClientMigratedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "ICS02ClientMigrated")
	if err != nil {
		return nil, err
	}
	return &ContractICS02ClientMigratedIterator{contract: _Contract.contract, event: "ICS02ClientMigrated", logs: logs, sub: sub}, nil
}

// WatchICS02ClientMigrated is a free log subscription operation binding the contract event 0x23c2e29d6ae84e79fa116b8afd6e28ddc1de7f473d3edb407fbd08093c3ed6bf.
//
// Solidity: event ICS02ClientMigrated(string clientId, (string,bytes[]) counterpartyInfo, address client)
func (_Contract *ContractFilterer) WatchICS02ClientMigrated(opts *bind.WatchOpts, sink chan<- *ContractICS02ClientMigrated) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "ICS02ClientMigrated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractICS02ClientMigrated)
				if err := _Contract.contract.UnpackLog(event, "ICS02ClientMigrated", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseICS02ClientMigrated is a log parse operation binding the contract event 0x23c2e29d6ae84e79fa116b8afd6e28ddc1de7f473d3edb407fbd08093c3ed6bf.
//
// Solidity: event ICS02ClientMigrated(string clientId, (string,bytes[]) counterpartyInfo, address client)
func (_Contract *ContractFilterer) ParseICS02ClientMigrated(log types.Log) (*ContractICS02ClientMigrated, error) {
	event := new(ContractICS02ClientMigrated)
	if err := _Contract.contract.UnpackLog(event, "ICS02ClientMigrated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractICS02ClientUpdatedIterator is returned from FilterICS02ClientUpdated and is used to iterate over the raw logs and unpacked data for ICS02ClientUpdated events raised by the Contract contract.
type ContractICS02ClientUpdatedIterator struct {
	Event *ContractICS02ClientUpdated // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractICS02ClientUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractICS02ClientUpdated)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractICS02ClientUpdated)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractICS02ClientUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractICS02ClientUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractICS02ClientUpdated represents a ICS02ClientUpdated event raised by the Contract contract.
type ContractICS02ClientUpdated struct {
	ClientId string
	Result   uint8
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterICS02ClientUpdated is a free log retrieval operation binding the contract event 0x87bbef2779889a19f0435ddca81fda94132c06ffddb0ea73def256307a293aef.
//
// Solidity: event ICS02ClientUpdated(string clientId, uint8 result)
func (_Contract *ContractFilterer) FilterICS02ClientUpdated(opts *bind.FilterOpts) (*ContractICS02ClientUpdatedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "ICS02ClientUpdated")
	if err != nil {
		return nil, err
	}
	return &ContractICS02ClientUpdatedIterator{contract: _Contract.contract, event: "ICS02ClientUpdated", logs: logs, sub: sub}, nil
}

// WatchICS02ClientUpdated is a free log subscription operation binding the contract event 0x87bbef2779889a19f0435ddca81fda94132c06ffddb0ea73def256307a293aef.
//
// Solidity: event ICS02ClientUpdated(string clientId, uint8 result)
func (_Contract *ContractFilterer) WatchICS02ClientUpdated(opts *bind.WatchOpts, sink chan<- *ContractICS02ClientUpdated) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "ICS02ClientUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractICS02ClientUpdated)
				if err := _Contract.contract.UnpackLog(event, "ICS02ClientUpdated", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseICS02ClientUpdated is a log parse operation binding the contract event 0x87bbef2779889a19f0435ddca81fda94132c06ffddb0ea73def256307a293aef.
//
// Solidity: event ICS02ClientUpdated(string clientId, uint8 result)
func (_Contract *ContractFilterer) ParseICS02ClientUpdated(log types.Log) (*ContractICS02ClientUpdated, error) {
	event := new(ContractICS02ClientUpdated)
	if err := _Contract.contract.UnpackLog(event, "ICS02ClientUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractICS02MisbehaviourSubmittedIterator is returned from FilterICS02MisbehaviourSubmitted and is used to iterate over the raw logs and unpacked data for ICS02MisbehaviourSubmitted events raised by the Contract contract.
type ContractICS02MisbehaviourSubmittedIterator struct {
	Event *ContractICS02MisbehaviourSubmitted // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractICS02MisbehaviourSubmittedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractICS02MisbehaviourSubmitted)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractICS02MisbehaviourSubmitted)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractICS02MisbehaviourSubmittedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractICS02MisbehaviourSubmittedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractICS02MisbehaviourSubmitted represents a ICS02MisbehaviourSubmitted event raised by the Contract contract.
type ContractICS02MisbehaviourSubmitted struct {
	ClientId string
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterICS02MisbehaviourSubmitted is a free log retrieval operation binding the contract event 0xa263f0a976b2937a51fd2e416491cf0ca724d5499fa870715929dfde4ee4a430.
//
// Solidity: event ICS02MisbehaviourSubmitted(string clientId)
func (_Contract *ContractFilterer) FilterICS02MisbehaviourSubmitted(opts *bind.FilterOpts) (*ContractICS02MisbehaviourSubmittedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "ICS02MisbehaviourSubmitted")
	if err != nil {
		return nil, err
	}
	return &ContractICS02MisbehaviourSubmittedIterator{contract: _Contract.contract, event: "ICS02MisbehaviourSubmitted", logs: logs, sub: sub}, nil
}

// WatchICS02MisbehaviourSubmitted is a free log subscription operation binding the contract event 0xa263f0a976b2937a51fd2e416491cf0ca724d5499fa870715929dfde4ee4a430.
//
// Solidity: event ICS02MisbehaviourSubmitted(string clientId)
func (_Contract *ContractFilterer) WatchICS02MisbehaviourSubmitted(opts *bind.WatchOpts, sink chan<- *ContractICS02MisbehaviourSubmitted) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "ICS02MisbehaviourSubmitted")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractICS02MisbehaviourSubmitted)
				if err := _Contract.contract.UnpackLog(event, "ICS02MisbehaviourSubmitted", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseICS02MisbehaviourSubmitted is a log parse operation binding the contract event 0xa263f0a976b2937a51fd2e416491cf0ca724d5499fa870715929dfde4ee4a430.
//
// Solidity: event ICS02MisbehaviourSubmitted(string clientId)
func (_Contract *ContractFilterer) ParseICS02MisbehaviourSubmitted(log types.Log) (*ContractICS02MisbehaviourSubmitted, error) {
	event := new(ContractICS02MisbehaviourSubmitted)
	if err := _Contract.contract.UnpackLog(event, "ICS02MisbehaviourSubmitted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractInitializedIterator is returned from FilterInitialized and is used to iterate over the raw logs and unpacked data for Initialized events raised by the Contract contract.
type ContractInitializedIterator struct {
	Event *ContractInitialized // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractInitializedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractInitialized)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractInitialized)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractInitializedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractInitializedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractInitialized represents a Initialized event raised by the Contract contract.
type ContractInitialized struct {
	Version uint64
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterInitialized is a free log retrieval operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_Contract *ContractFilterer) FilterInitialized(opts *bind.FilterOpts) (*ContractInitializedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Initialized")
	if err != nil {
		return nil, err
	}
	return &ContractInitializedIterator{contract: _Contract.contract, event: "Initialized", logs: logs, sub: sub}, nil
}

// WatchInitialized is a free log subscription operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_Contract *ContractFilterer) WatchInitialized(opts *bind.WatchOpts, sink chan<- *ContractInitialized) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Initialized")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractInitialized)
				if err := _Contract.contract.UnpackLog(event, "Initialized", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseInitialized is a log parse operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_Contract *ContractFilterer) ParseInitialized(log types.Log) (*ContractInitialized, error) {
	event := new(ContractInitialized)
	if err := _Contract.contract.UnpackLog(event, "Initialized", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractNoopIterator is returned from FilterNoop and is used to iterate over the raw logs and unpacked data for Noop events raised by the Contract contract.
type ContractNoopIterator struct {
	Event *ContractNoop // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractNoopIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractNoop)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractNoop)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractNoopIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractNoopIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractNoop represents a Noop event raised by the Contract contract.
type ContractNoop struct {
	Raw types.Log // Blockchain specific contextual infos
}

// FilterNoop is a free log retrieval operation binding the contract event 0xd08bf58b0e4eec5bfc697a4fdbb6839057fbf4dd06f1b1ce07445c0e5a654caf.
//
// Solidity: event Noop()
func (_Contract *ContractFilterer) FilterNoop(opts *bind.FilterOpts) (*ContractNoopIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Noop")
	if err != nil {
		return nil, err
	}
	return &ContractNoopIterator{contract: _Contract.contract, event: "Noop", logs: logs, sub: sub}, nil
}

// WatchNoop is a free log subscription operation binding the contract event 0xd08bf58b0e4eec5bfc697a4fdbb6839057fbf4dd06f1b1ce07445c0e5a654caf.
//
// Solidity: event Noop()
func (_Contract *ContractFilterer) WatchNoop(opts *bind.WatchOpts, sink chan<- *ContractNoop) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Noop")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractNoop)
				if err := _Contract.contract.UnpackLog(event, "Noop", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseNoop is a log parse operation binding the contract event 0xd08bf58b0e4eec5bfc697a4fdbb6839057fbf4dd06f1b1ce07445c0e5a654caf.
//
// Solidity: event Noop()
func (_Contract *ContractFilterer) ParseNoop(log types.Log) (*ContractNoop, error) {
	event := new(ContractNoop)
	if err := _Contract.contract.UnpackLog(event, "Noop", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractSendPacketIterator is returned from FilterSendPacket and is used to iterate over the raw logs and unpacked data for SendPacket events raised by the Contract contract.
type ContractSendPacketIterator struct {
	Event *ContractSendPacket // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractSendPacketIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractSendPacket)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractSendPacket)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractSendPacketIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractSendPacketIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractSendPacket represents a SendPacket event raised by the Contract contract.
type ContractSendPacket struct {
	ClientId common.Hash
	Sequence *big.Int
	Packet   IICS26RouterMsgsPacket
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterSendPacket is a free log retrieval operation binding the contract event 0xab3a4458a269be61dfa43faa33aa7b1f5d570716f83ad078bc2ba5dab039abae.
//
// Solidity: event SendPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet)
func (_Contract *ContractFilterer) FilterSendPacket(opts *bind.FilterOpts, clientId []string, sequence []*big.Int) (*ContractSendPacketIterator, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "SendPacket", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return &ContractSendPacketIterator{contract: _Contract.contract, event: "SendPacket", logs: logs, sub: sub}, nil
}

// WatchSendPacket is a free log subscription operation binding the contract event 0xab3a4458a269be61dfa43faa33aa7b1f5d570716f83ad078bc2ba5dab039abae.
//
// Solidity: event SendPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet)
func (_Contract *ContractFilterer) WatchSendPacket(opts *bind.WatchOpts, sink chan<- *ContractSendPacket, clientId []string, sequence []*big.Int) (event.Subscription, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "SendPacket", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractSendPacket)
				if err := _Contract.contract.UnpackLog(event, "SendPacket", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseSendPacket is a log parse operation binding the contract event 0xab3a4458a269be61dfa43faa33aa7b1f5d570716f83ad078bc2ba5dab039abae.
//
// Solidity: event SendPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet)
func (_Contract *ContractFilterer) ParseSendPacket(log types.Log) (*ContractSendPacket, error) {
	event := new(ContractSendPacket)
	if err := _Contract.contract.UnpackLog(event, "SendPacket", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractTimeoutPacketIterator is returned from FilterTimeoutPacket and is used to iterate over the raw logs and unpacked data for TimeoutPacket events raised by the Contract contract.
type ContractTimeoutPacketIterator struct {
	Event *ContractTimeoutPacket // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractTimeoutPacketIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractTimeoutPacket)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractTimeoutPacket)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractTimeoutPacketIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractTimeoutPacketIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractTimeoutPacket represents a TimeoutPacket event raised by the Contract contract.
type ContractTimeoutPacket struct {
	ClientId common.Hash
	Sequence *big.Int
	Packet   IICS26RouterMsgsPacket
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterTimeoutPacket is a free log retrieval operation binding the contract event 0x01e5ed58494819ef3f6480dd08e433b7c08ed75c7abdf2c22c6f04b71340a168.
//
// Solidity: event TimeoutPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet)
func (_Contract *ContractFilterer) FilterTimeoutPacket(opts *bind.FilterOpts, clientId []string, sequence []*big.Int) (*ContractTimeoutPacketIterator, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "TimeoutPacket", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return &ContractTimeoutPacketIterator{contract: _Contract.contract, event: "TimeoutPacket", logs: logs, sub: sub}, nil
}

// WatchTimeoutPacket is a free log subscription operation binding the contract event 0x01e5ed58494819ef3f6480dd08e433b7c08ed75c7abdf2c22c6f04b71340a168.
//
// Solidity: event TimeoutPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet)
func (_Contract *ContractFilterer) WatchTimeoutPacket(opts *bind.WatchOpts, sink chan<- *ContractTimeoutPacket, clientId []string, sequence []*big.Int) (event.Subscription, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "TimeoutPacket", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractTimeoutPacket)
				if err := _Contract.contract.UnpackLog(event, "TimeoutPacket", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseTimeoutPacket is a log parse operation binding the contract event 0x01e5ed58494819ef3f6480dd08e433b7c08ed75c7abdf2c22c6f04b71340a168.
//
// Solidity: event TimeoutPacket(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet)
func (_Contract *ContractFilterer) ParseTimeoutPacket(log types.Log) (*ContractTimeoutPacket, error) {
	event := new(ContractTimeoutPacket)
	if err := _Contract.contract.UnpackLog(event, "TimeoutPacket", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractUpgradedIterator is returned from FilterUpgraded and is used to iterate over the raw logs and unpacked data for Upgraded events raised by the Contract contract.
type ContractUpgradedIterator struct {
	Event *ContractUpgraded // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractUpgradedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractUpgraded)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractUpgraded)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractUpgradedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractUpgradedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractUpgraded represents a Upgraded event raised by the Contract contract.
type ContractUpgraded struct {
	Implementation common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterUpgraded is a free log retrieval operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_Contract *ContractFilterer) FilterUpgraded(opts *bind.FilterOpts, implementation []common.Address) (*ContractUpgradedIterator, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return &ContractUpgradedIterator{contract: _Contract.contract, event: "Upgraded", logs: logs, sub: sub}, nil
}

// WatchUpgraded is a free log subscription operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_Contract *ContractFilterer) WatchUpgraded(opts *bind.WatchOpts, sink chan<- *ContractUpgraded, implementation []common.Address) (event.Subscription, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractUpgraded)
				if err := _Contract.contract.UnpackLog(event, "Upgraded", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseUpgraded is a log parse operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_Contract *ContractFilterer) ParseUpgraded(log types.Log) (*ContractUpgraded, error) {
	event := new(ContractUpgraded)
	if err := _Contract.contract.UnpackLog(event, "Upgraded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractWriteAcknowledgementIterator is returned from FilterWriteAcknowledgement and is used to iterate over the raw logs and unpacked data for WriteAcknowledgement events raised by the Contract contract.
type ContractWriteAcknowledgementIterator struct {
	Event *ContractWriteAcknowledgement // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractWriteAcknowledgementIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractWriteAcknowledgement)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractWriteAcknowledgement)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractWriteAcknowledgementIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractWriteAcknowledgementIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractWriteAcknowledgement represents a WriteAcknowledgement event raised by the Contract contract.
type ContractWriteAcknowledgement struct {
	ClientId         common.Hash
	Sequence         *big.Int
	Packet           IICS26RouterMsgsPacket
	Acknowledgements [][]byte
	Raw              types.Log // Blockchain specific contextual infos
}

// FilterWriteAcknowledgement is a free log retrieval operation binding the contract event 0x76765590e2b799b0506100f8a6610cfecab2c71e8e1f8aa981b099aff0dfdb74.
//
// Solidity: event WriteAcknowledgement(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet, bytes[] acknowledgements)
func (_Contract *ContractFilterer) FilterWriteAcknowledgement(opts *bind.FilterOpts, clientId []string, sequence []*big.Int) (*ContractWriteAcknowledgementIterator, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "WriteAcknowledgement", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return &ContractWriteAcknowledgementIterator{contract: _Contract.contract, event: "WriteAcknowledgement", logs: logs, sub: sub}, nil
}

// WatchWriteAcknowledgement is a free log subscription operation binding the contract event 0x76765590e2b799b0506100f8a6610cfecab2c71e8e1f8aa981b099aff0dfdb74.
//
// Solidity: event WriteAcknowledgement(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet, bytes[] acknowledgements)
func (_Contract *ContractFilterer) WatchWriteAcknowledgement(opts *bind.WatchOpts, sink chan<- *ContractWriteAcknowledgement, clientId []string, sequence []*big.Int) (event.Subscription, error) {

	var clientIdRule []interface{}
	for _, clientIdItem := range clientId {
		clientIdRule = append(clientIdRule, clientIdItem)
	}
	var sequenceRule []interface{}
	for _, sequenceItem := range sequence {
		sequenceRule = append(sequenceRule, sequenceItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "WriteAcknowledgement", clientIdRule, sequenceRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractWriteAcknowledgement)
				if err := _Contract.contract.UnpackLog(event, "WriteAcknowledgement", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseWriteAcknowledgement is a log parse operation binding the contract event 0x76765590e2b799b0506100f8a6610cfecab2c71e8e1f8aa981b099aff0dfdb74.
//
// Solidity: event WriteAcknowledgement(string indexed clientId, uint256 indexed sequence, (uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet, bytes[] acknowledgements)
func (_Contract *ContractFilterer) ParseWriteAcknowledgement(log types.Log) (*ContractWriteAcknowledgement, error) {
	event := new(ContractWriteAcknowledgement)
	if err := _Contract.contract.UnpackLog(event, "WriteAcknowledgement", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
