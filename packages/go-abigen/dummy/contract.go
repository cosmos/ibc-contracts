// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package dummy

import (
	"errors"
	"math/big"
	"strings"

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
)

// IICS02ClientMsgsHeight is an auto generated low-level Go binding around an user-defined struct.
type IICS02ClientMsgsHeight struct {
	RevisionNumber uint64
	RevisionHeight uint64
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
	ABI: "[{\"type\":\"function\",\"name\":\"getClientState\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"misbehaviour\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"updateMsg\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"enumILightClientMsgs.UpdateResult\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"verifyMembership\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"verifyNonMembership\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyNonMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"error\",\"name\":\"MembershipExists\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"MissingMembership\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UnknownHeight\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ValueMismatch\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ZeroTimestamp\",\"inputs\":[]}]",
	Bin: "0x60808060405234601557610ab7908161001a8239f35b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c9081630bece356146102ac575080634d6d9ffb1461020d578063682ed5f0146100d2578063ddba6537146100c25763ef913a4b14610053575f80fd5b346100be575f6003193601126100be576100ba60405167ffffffffffffffff5f54818116602084015260401c16604082015267ffffffffffffffff600154166060820152606081526100a66080826107a9565b604051918291602083526020830190610768565b0390f35b5f80fd5b346100be576100d036610717565b005b346100be5760206003193601126100be5760043567ffffffffffffffff81116100be578036039060a06003198301126100be57602481019161012d61011684610929565b93610127606485018560040161085d565b9161098f565b5f52600360205260405f20549182156101e5577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdd608482013592018212156100be570160048101359067ffffffffffffffff82116100be5760240181360381136100be5761019c9136916107f9565b60208151910120036101bd5760209067ffffffffffffffff60405191168152f35b7fdd8e4af7000000000000000000000000000000000000000000000000000000005f5260045ffd5b7ff6b593a3000000000000000000000000000000000000000000000000000000005f5260045ffd5b346100be5760206003193601126100be5760043567ffffffffffffffff81116100be57608060031982360301126100be5761025f6024820161012761025182610929565b93606481019060040161085d565b5f52600360205260405f20546102845760209067ffffffffffffffff60405191168152f35b7f679fc91d000000000000000000000000000000000000000000000000000000005f5260045ffd5b346100be576102ba36610717565b8101906020818303126100be5780359067ffffffffffffffff82116100be570191828203608081126100be57606082019082821067ffffffffffffffff8311176106ea576040918252126100be576040516103148161078d565b61031d846107cc565b815261032b602085016107cc565b6020820152815261033e604084016107cc565b926020820193845260608101359067ffffffffffffffff82116100be57019180601f840112156100be57823592610374846107e1565b9361038260405195866107a9565b80855260208086019160051b830101918383116100be5760208101915b8383106105ef578787876040810191825267ffffffffffffffff835116156105c757519167ffffffffffffffff80845116916103e3602086019383855116906108b1565b82825116905f52600260205260405f20907fffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000825416179055511680602060405161042c8161078d565b868152015267ffffffffffffffff8451167fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000005f5416175f5581517fffffffffffffffffffffffffffffffff0000000000000000ffffffffffffffff6fffffffffffffffff00000000000000005f549260401b169116175f557fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000060015416176001555f5b825180518210156105bd576020908260051b010151805160405160208101916040820160208452815180915260608301602060608360051b8601019301915f5b81811061057657505050506001949392826105386020946105599403601f1981018352826107a9565b51902067ffffffffffffffff89511667ffffffffffffffff885116906108e3565b91015160208151910120905f52600360205260405f2055016104cf565b909192936020806105b1837fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa08a600196030188528851610768565b9601940192910161050f565b60206040515f8152f35b7fda16d767000000000000000000000000000000000000000000000000000000005f5260045ffd5b823567ffffffffffffffff81116100be578201906040601f1983880301126100be576040519061061e8261078d565b602083013567ffffffffffffffff81116100be5760209084010187601f820112156100be57803561064e816107e1565b9161065c60405193846107a9565b81835260208084019260051b820101918a83116100be5760208201905b8382106106bc5750505050825260408301359167ffffffffffffffff83116100be576106ad8860208096958196010161083f565b8382015281520192019161039f565b813567ffffffffffffffff81116100be576020916106df8e84809488010161083f565b815201910190610679565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b9060206003198301126100be5760043567ffffffffffffffff81116100be57826023820112156100be5780600401359267ffffffffffffffff84116100be57602484830101116100be576024019190565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b6040810190811067ffffffffffffffff8211176106ea57604052565b90601f601f19910116810190811067ffffffffffffffff8211176106ea57604052565b359067ffffffffffffffff821682036100be57565b67ffffffffffffffff81116106ea5760051b60200190565b92919267ffffffffffffffff82116106ea57604051916108236020601f19601f84011601846107a9565b8294818452818301116100be578281602093845f960137010152565b9080601f830112156100be5781602061085a933591016107f9565b90565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1813603018212156100be570180359067ffffffffffffffff82116100be57602001918160051b360383136100be57565b9067ffffffffffffffff60405191816020840194168452166040820152604081526108dd6060826107a9565b51902090565b9167ffffffffffffffff604051928160208501951685521660408301526060820152606081526108dd6080826107a9565b3567ffffffffffffffff811681036100be5790565b61094890610942602061093b83610914565b9201610914565b906108b1565b5f52600260205267ffffffffffffffff60405f20541680156109675790565b7f8da291bd000000000000000000000000000000000000000000000000000000005f5260045ffd5b909291926109a860206109a184610914565b9301610914565b90604051946020860191816040880160208552526060870160608360051b89010192825f907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1813603015b838310610a215750505050505094610a188161085a969703601f1981018352826107a9565b519020916108e3565b9091929394957fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa08c82030186528635828112156100be578301906020823592019167ffffffffffffffff81116100be5780360383136100be57602082601f19601f8480600198869897879852868601375f848201860152011601019801960194930191906109f356fea164736f6c634300081c000a",
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

// GetClientState is a free data retrieval call binding the contract method 0xef913a4b.
//
// Solidity: function getClientState() view returns(bytes)
func (_Contract *ContractCaller) GetClientState(opts *bind.CallOpts) ([]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getClientState")

	if err != nil {
		return *new([]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([]byte)).(*[]byte)

	return out0, err

}

// GetClientState is a free data retrieval call binding the contract method 0xef913a4b.
//
// Solidity: function getClientState() view returns(bytes)
func (_Contract *ContractSession) GetClientState() ([]byte, error) {
	return _Contract.Contract.GetClientState(&_Contract.CallOpts)
}

// GetClientState is a free data retrieval call binding the contract method 0xef913a4b.
//
// Solidity: function getClientState() view returns(bytes)
func (_Contract *ContractCallerSession) GetClientState() ([]byte, error) {
	return _Contract.Contract.GetClientState(&_Contract.CallOpts)
}

// Misbehaviour is a paid mutator transaction binding the contract method 0xddba6537.
//
// Solidity: function misbehaviour(bytes ) returns()
func (_Contract *ContractTransactor) Misbehaviour(opts *bind.TransactOpts, arg0 []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "misbehaviour", arg0)
}

// Misbehaviour is a paid mutator transaction binding the contract method 0xddba6537.
//
// Solidity: function misbehaviour(bytes ) returns()
func (_Contract *ContractSession) Misbehaviour(arg0 []byte) (*types.Transaction, error) {
	return _Contract.Contract.Misbehaviour(&_Contract.TransactOpts, arg0)
}

// Misbehaviour is a paid mutator transaction binding the contract method 0xddba6537.
//
// Solidity: function misbehaviour(bytes ) returns()
func (_Contract *ContractTransactorSession) Misbehaviour(arg0 []byte) (*types.Transaction, error) {
	return _Contract.Contract.Misbehaviour(&_Contract.TransactOpts, arg0)
}

// UpdateClient is a paid mutator transaction binding the contract method 0x0bece356.
//
// Solidity: function updateClient(bytes updateMsg) returns(uint8)
func (_Contract *ContractTransactor) UpdateClient(opts *bind.TransactOpts, updateMsg []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "updateClient", updateMsg)
}

// UpdateClient is a paid mutator transaction binding the contract method 0x0bece356.
//
// Solidity: function updateClient(bytes updateMsg) returns(uint8)
func (_Contract *ContractSession) UpdateClient(updateMsg []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpdateClient(&_Contract.TransactOpts, updateMsg)
}

// UpdateClient is a paid mutator transaction binding the contract method 0x0bece356.
//
// Solidity: function updateClient(bytes updateMsg) returns(uint8)
func (_Contract *ContractTransactorSession) UpdateClient(updateMsg []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpdateClient(&_Contract.TransactOpts, updateMsg)
}

// VerifyMembership is a paid mutator transaction binding the contract method 0x682ed5f0.
//
// Solidity: function verifyMembership((bytes,(uint64,uint64),bytes[],bytes) msg_) returns(uint256)
func (_Contract *ContractTransactor) VerifyMembership(opts *bind.TransactOpts, msg_ ILightClientMsgsMsgVerifyMembership) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "verifyMembership", msg_)
}

// VerifyMembership is a paid mutator transaction binding the contract method 0x682ed5f0.
//
// Solidity: function verifyMembership((bytes,(uint64,uint64),bytes[],bytes) msg_) returns(uint256)
func (_Contract *ContractSession) VerifyMembership(msg_ ILightClientMsgsMsgVerifyMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyMembership(&_Contract.TransactOpts, msg_)
}

// VerifyMembership is a paid mutator transaction binding the contract method 0x682ed5f0.
//
// Solidity: function verifyMembership((bytes,(uint64,uint64),bytes[],bytes) msg_) returns(uint256)
func (_Contract *ContractTransactorSession) VerifyMembership(msg_ ILightClientMsgsMsgVerifyMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyMembership(&_Contract.TransactOpts, msg_)
}

// VerifyNonMembership is a paid mutator transaction binding the contract method 0x4d6d9ffb.
//
// Solidity: function verifyNonMembership((bytes,(uint64,uint64),bytes[]) msg_) returns(uint256)
func (_Contract *ContractTransactor) VerifyNonMembership(opts *bind.TransactOpts, msg_ ILightClientMsgsMsgVerifyNonMembership) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "verifyNonMembership", msg_)
}

// VerifyNonMembership is a paid mutator transaction binding the contract method 0x4d6d9ffb.
//
// Solidity: function verifyNonMembership((bytes,(uint64,uint64),bytes[]) msg_) returns(uint256)
func (_Contract *ContractSession) VerifyNonMembership(msg_ ILightClientMsgsMsgVerifyNonMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyNonMembership(&_Contract.TransactOpts, msg_)
}

// VerifyNonMembership is a paid mutator transaction binding the contract method 0x4d6d9ffb.
//
// Solidity: function verifyNonMembership((bytes,(uint64,uint64),bytes[]) msg_) returns(uint256)
func (_Contract *ContractTransactorSession) VerifyNonMembership(msg_ ILightClientMsgsMsgVerifyNonMembership) (*types.Transaction, error) {
	return _Contract.Contract.VerifyNonMembership(&_Contract.TransactOpts, msg_)
}
