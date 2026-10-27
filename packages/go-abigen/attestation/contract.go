// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package attestation

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
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"attestorAddresses\",\"type\":\"address[]\",\"internalType\":\"address[]\"},{\"name\":\"minRequiredSigs\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"initialHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"initialTimestampSeconds\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"ics26Router\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"ICS26_ROUTER\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getAttestationSet\",\"inputs\":[],\"outputs\":[{\"name\":\"attestorAddresses\",\"type\":\"address[]\",\"internalType\":\"address[]\"},{\"name\":\"minRequiredSigs\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getClientState\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getConsensusTimestamp\",\"inputs\":[{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"misbehaviour\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"pause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"paused\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"unpause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"updateMsg\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"enumILightClientMsgs.UpdateResult\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"verifyMembership\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"verifyNonMembership\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyNonMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"event\",\"name\":\"Paused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Unpaused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"BadQuorum\",\"inputs\":[{\"name\":\"minRequired\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"attestationCount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ConsensusTimestampNotFound\",\"inputs\":[{\"name\":\"height\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"DuplicateSigner\",\"inputs\":[{\"name\":\"signer\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignature\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignatureLength\",\"inputs\":[{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignatureS\",\"inputs\":[{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"EmptyPackets\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EmptySignatures\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EmptyValue\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EnforcedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ExpectedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FeatureNotSupported\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FrozenClientState\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"HeightMismatch\",\"inputs\":[{\"name\":\"expected\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"provided\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"InvalidPathLength\",\"inputs\":[{\"name\":\"expectedLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"providedLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidSignatureLength\",\"inputs\":[{\"name\":\"signature\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"InvalidState\",\"inputs\":[{\"name\":\"height\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"LightClientUnauthorizedCaller\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"LightClientUnauthorizedRole\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"roleId\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"LightClientZeroICS26Router\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NoAttestors\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotMember\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotNonMember\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SignatureInvalid\",\"inputs\":[{\"name\":\"signature\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"ThresholdNotMet\",\"inputs\":[{\"name\":\"validSigners\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"minRequired\",\"type\":\"uint8\",\"internalType\":\"uint8\"}]},{\"type\":\"error\",\"name\":\"UnknownSigner\",\"inputs\":[{\"name\":\"signer\",\"type\":\"address\",\"internalType\":\"address\"}]}]",
	Bin: "0x60a06040523461033857611d088038038061001981610354565b928339810160a0828203126103385781516001600160401b0381116103385782019181601f84011215610338578251916001600160401b0383116102c4578260051b936020610069818701610354565b8095815201906020829682010192831161033857602001905b82821061033c5750505060208101519260ff8416809403610338576100a96040830161038d565b906100c260806100bb6060860161038d565b9401610379565b6001600160a01b038116156103295760805283511561031a57841515806102ee575b845190156102d8575060405194608086016001600160401b038111878210176102c457604052848652602086019081526060604087019360018060401b03169687855201915f835285519060018060401b0382116102c4576801000000000000000082116102c45760015482600155808310610299575b5060015f5260205f205f5b83811061027c575050505060ff90511669ff00000000000000000060025493610100600160481b03905160081b169251151560481b169260018060501b0319161717176002555f5b825181101561020e5760018060a01b0360208260051b8501015116805f52600360205260ff60405f2054166101fc57906001915f52600360205260405f208260ff19825416179055016101ae565b637010e27960e11b5f5260045260245ffd5b505f838152600460205260409081902080546001600160401b0319166001600160401b03909316929092179091555161196690816103a282396080518181816101680152818161020b0152818161030f0152818161051f01528181610624015281816106a6015261088c0152f35b82516001600160a01b031681830155602090920191600101610166565b60015f52828060205f20019103905f5b8281106102b757505061015b565b5f828201556001016102a9565b634e487b7160e01b5f52604160045260245ffd5b856365846b7f60e01b5f5260045260245260445ffd5b5083515f19860160ff81116103065760ff16106100e4565b634e487b7160e01b5f52601160045260245ffd5b6343d0a89360e11b5f5260045ffd5b634c16a43960e11b5f5260045ffd5b5f80fd5b6020809161034984610379565b815201910190610082565b6040519190601f01601f191682016001600160401b038111838210176102c457604052565b51906001600160a01b038216820361033857565b51906001600160401b03821682036103385756fe60806040526004361015610011575f80fd5b5f3560e01c80630bece3561461084a5780633f4ba83a146106535780634d6d9ffb146105bd5780635832611b146105775780635c975abb14610556578063682ed5f0146104b85780638456cb59146102bc578063a845a7f31461022f578063c20f00e1146101df578063ddba6537146101305763ef913a4b14610092575f80fd5b3461012c575f60031936011261012c576101286040516020808201526080604082015261011c816100c560c0820161143c565b60ff600254818116606085015267ffffffffffffffff8160081c16608085015260481c16151560a0830152037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe08101835282610a24565b60405191829182610949565b0390f35b5f80fd5b3461012c5761013e366108f8565b505060ff60025460481c166101b75761018f3373ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000163314610991565b7fda81d7c2000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f928b1233000000000000000000000000000000000000000000000000000000005f5260045ffd5b3461012c575f60031936011261012c57602060405173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000168152f35b3461012c575f60031936011261012c5760ff6002541660405161025c816102558161143c565b0382610a24565b60405190604082019260408352815180945260206060840192015f945b80861061028e57505082935060208301520390f35b909260208060019273ffffffffffffffffffffffffffffffffffffffff875116815201940195019490610279565b3461012c575f60031936011261012c576040517fbf7e214f00000000000000000000000000000000000000000000000000000000815260208160048173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000165afa90811561047e575f91610489575b50604080517fd1f856ee00000000000000000000000000000000000000000000000000000000815260026004820152336024820152918290604490829073ffffffffffffffffffffffffffffffffffffffff165afa801561047e575f915f9161044d575b508161043e575b501561040d576103b861148b565b60017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff005f5416175f557f62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a2586020604051338152a1005b7faf1ecfa5000000000000000000000000000000000000000000000000000000005f5233600452600260245260445ffd5b63ffffffff91501615816103aa565b9050610471915060403d604011610477575b6104698183610a24565b810190610e83565b826103a3565b503d61045f565b6040513d5f823e3d90fd5b6104ab915060203d6020116104b1575b6104a38183610a24565b810190610e57565b8161033f565b503d610499565b3461012c57602060031936011261012c5760043567ffffffffffffffff811161012c5760a0600319823603011261012c5760ff60025460481c166101b75761054e60209161050461148b565b6105463373ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000163314610991565b6004016112df565b604051908152f35b3461012c575f60031936011261012c57602060ff5f54166040519015158152f35b3461012c57602060031936011261012c5760043567ffffffffffffffff811680910361012c575f526004602052602067ffffffffffffffff60405f205416604051908152f35b3461012c57602060031936011261012c5760043567ffffffffffffffff811161012c576080600319823603011261012c5760ff60025460481c166101b75761054e60209161060961148b565b61064b3373ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000163314610991565b600401611109565b3461012c575f60031936011261012c576040517fbf7e214f00000000000000000000000000000000000000000000000000000000815260208160048173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000165afa90811561047e575f9161082b575b50604080517fd1f856ee00000000000000000000000000000000000000000000000000000000815260036004820152336024820152918290604490829073ffffffffffffffffffffffffffffffffffffffff165afa801561047e575f915f9161080a575b50816107fb575b50156107ca575f5460ff8116156107a2577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00165f557f5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa6020604051338152a1005b7f8dfc202b000000000000000000000000000000000000000000000000000000005f5260045ffd5b7faf1ecfa5000000000000000000000000000000000000000000000000000000005f5233600452600360245260445ffd5b63ffffffff9150161581610741565b9050610825915060403d604011610477576104698183610a24565b8261073a565b610844915060203d6020116104b1576104a38183610a24565b816106d6565b3461012c57610858366108f8565b60ff60025460481c166101b7576108b89161087161148b565b6108b33373ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000163314610991565b610c02565b60405160038210156108cb576020918152f35b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602160045260245ffd5b90602060031983011261012c5760043567ffffffffffffffff811161012c578260238201121561012c5780600401359267ffffffffffffffff841161012c576024848301011161012c576024019190565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f602060409481855280519182918282880152018686015e5f8582860101520116010190565b156109995750565b73ffffffffffffffffffffffffffffffffffffffff907f091c7efd000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b6040810190811067ffffffffffffffff8211176109f757604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b90601f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0910116810190811067ffffffffffffffff8211176109f757604052565b92919267ffffffffffffffff82116109f75760405191610aad60207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f8401160184610a24565b82948184528183011161012c578281602093845f960137010152565b9080601f8301121561012c57816020610ae493359101610a65565b90565b67ffffffffffffffff81116109f75760051b60200190565b60208183031261012c5780359067ffffffffffffffff821161012c57019060408282031261012c5760405191610b34836109db565b803567ffffffffffffffff811161012c5782610b51918301610ac9565b835260208101359067ffffffffffffffff821161012c57019080601f8301121561012c578135610b8081610ae7565b92610b8e6040519485610a24565b81845260208085019260051b8201019183831161012c5760208201905b838210610bbf575050505050602082015290565b813567ffffffffffffffff811161012c57602091610be287848094880101610ac9565b815201910190610bab565b519067ffffffffffffffff8216820361012c57565b610c0e91810190610aff565b60205f818351604051918183925191829101835e8101838152039060025afa1561047e5760205f8051604051838101917f01000000000000000000000000000000000000000000000000000000000000008352602182015260218152610c75604182610a24565b604051918291518091835e8101838152039060025afa1561047e57610ca05f51602083015190611544565b5160408180518101031261012c57604051610cba816109db565b610cd56040610ccb60208501610bed565b9384845201610bed565b67ffffffffffffffff60208301938285521680151580610e44575b15610e0a57505067ffffffffffffffff8151165f52600460205267ffffffffffffffff60405f20541680610db9575067ffffffffffffffff9051918183169260025490838260081c168511610d7c575b50505116905f52600460205260405f20907fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000008254161790555f90565b68ffffffffffffffff007fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000ff9160081b169116176002555f80610d40565b9167ffffffffffffffff9150511603610dd157600290565b69010000000000000000007fffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffffff6002541617600255600190565b67ffffffffffffffff92507fdcbc5460000000000000000000000000000000000000000000000000000000005f526004521660245260445ffd5b5067ffffffffffffffff82161515610cf0565b9081602091031261012c575173ffffffffffffffffffffffffffffffffffffffff8116810361012c5790565b919082604091031261012c578151801515810361012c5760209092015163ffffffff8116810361012c5790565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18136030182121561012c570180359067ffffffffffffffff821161012c57602001918160051b3603831361012c57565b3567ffffffffffffffff8116810361012c5790565b15610f215750565b67ffffffffffffffff907ff7caaa5c000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18136030182121561012c570180359067ffffffffffffffff821161012c5760200191813603831361012c57565b60208183031261012c5780519067ffffffffffffffff821161012c57019060408282031261012c5760405191610fdd836109db565b610fe681610bed565b835260208101519067ffffffffffffffff821161012c570181601f8201121561012c5780519061101582610ae7565b926110236040519485610a24565b82845260208085019360061b8301019181831161012c57602001925b8284106110525750505050602082015290565b60408483031261012c576020604091825161106c816109db565b86518152828701518382015281520193019261103f565b1561108c575050565b9067ffffffffffffffff80927fc7441689000000000000000000000000000000000000000000000000000000005f52166004521660245260445ffd5b80518210156110dc5760209160051b010190565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b60608101906111188282610eb0565b905060016111268484610eb0565b929050036112af57506111cc61113e60408301610f04565b9267ffffffffffffffff841692835f5260046020526111c767ffffffffffffffff60405f2054169461117287871515610f19565b6111b261118a6111828580610f57565b810190610aff565b6111a261119782516114be565b602083015190611544565b5160208082518301019101610fa8565b9667ffffffffffffffff885116918214611083565b610eb0565b156110dc576111e76111e082602093610f57565b3691610a65565b818151910120920180515115611287575f5b8151805182101561125f5761120f8286926110c8565b51511461121e576001016111f9565b602092935061122d91516110c8565b5101516112375790565b7f9a960b4f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f291fc442000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f1e6c84da000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f88b3170e000000000000000000000000000000000000000000000000000000005f52600160045260245260445ffd5b608081016112ed8183610f57565b9050156114145760608201906113038284610eb0565b905060016113118486610eb0565b929050036112af575061132660408401610f04565b9061138867ffffffffffffffff831693845f52600460205261138267ffffffffffffffff60405f2054169561135d86881515610f19565b61136d61118a6111828a80610f57565b9567ffffffffffffffff875116918214611083565b85610eb0565b156110dc576020916113a06111e0836113ac94610f57565b83815191012095610f57565b908092918101031261012c576020903591019081515115611287575f5b8251805182101561125f576113df8287926110c8565b515114806113fc575b6113f4576001016113c9565b505050905090565b5081602061140b8386516110c8565b510151146113e8565b7f1208b21b000000000000000000000000000000000000000000000000000000005f5260045ffd5b602060015491828152019060015f5260205f20905f5b81811061145f5750505090565b825473ffffffffffffffffffffffffffffffffffffffff16845260209093019260019283019201611452565b60ff5f541661149657565b7fd93c0665000000000000000000000000000000000000000000000000000000005f5260045ffd5b5f60208092604051918183925191829101835e8101838152039060025afa1561047e5760205f8051604051838101917f02000000000000000000000000000000000000000000000000000000000000008352602182015260218152611524604182610a24565b604051918291518091835e8101838152039060025afa1561047e575f5190565b9190918251156117bd57825160ff60025416907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82019060ff82116117905760ff8651921610156117625750508251927fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe06115d76115c186610ae7565b956115cf6040519788610a24565b808752610ae7565b013660208601375f5b815181101561175b576115f381836110c8565b5160418151036117255773ffffffffffffffffffffffffffffffffffffffff61162861161f83876117e5565b9092919261181f565b169081156116eb5750805f52600360205260ff60405f205416156116c0575f5b82811061166557509060019161165e82886110c8565b52016115e0565b8173ffffffffffffffffffffffffffffffffffffffff611685838a6110c8565b51161461169457600101611648565b507fe021c4f2000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7fd7e8a2c2000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b611721906040519182917f9e3c1b3d00000000000000000000000000000000000000000000000000000000835260048301610949565b0390fd5b611721906040519182917f2ee17a3d00000000000000000000000000000000000000000000000000000000835260048301610949565b5050509050565b7f378b0805000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b7f301b38b6000000000000000000000000000000000000000000000000000000005f5260045ffd5b81519190604183036118155761180e9250602082015190606060408401519301515f1a906118ca565b9192909190565b50505f9160029190565b60048110156108cb5780611831575050565b60018103611861577ff645eedf000000000000000000000000000000000000000000000000000000005f5260045ffd5b6002810361189557507ffce698f7000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b60031461189f5750565b7fd78bce0c000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b91907f7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0841161194e579160209360809260ff5f9560405194855216868401526040830152606082015282805260015afa1561047e575f5173ffffffffffffffffffffffffffffffffffffffff81161561194457905f905f90565b505f906001905f90565b5050505f916003919056fea164736f6c6343000825000a",
}

// ContractABI is the input ABI used to generate the binding from.
// Deprecated: Use ContractMetaData.ABI instead.
var ContractABI = ContractMetaData.ABI

// ContractBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ContractMetaData.Bin instead.
var ContractBin = ContractMetaData.Bin

// DeployContract deploys a new Ethereum contract, binding an instance of Contract to it.
func DeployContract(auth *bind.TransactOpts, backend bind.ContractBackend, attestorAddresses []common.Address, minRequiredSigs uint8, initialHeight uint64, initialTimestampSeconds uint64, ics26Router common.Address) (common.Address, *types.Transaction, *Contract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ContractBin), backend, attestorAddresses, minRequiredSigs, initialHeight, initialTimestampSeconds, ics26Router)
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

// ICS26ROUTER is a free data retrieval call binding the contract method 0xc20f00e1.
//
// Solidity: function ICS26_ROUTER() view returns(address)
func (_Contract *ContractCaller) ICS26ROUTER(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "ICS26_ROUTER")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ICS26ROUTER is a free data retrieval call binding the contract method 0xc20f00e1.
//
// Solidity: function ICS26_ROUTER() view returns(address)
func (_Contract *ContractSession) ICS26ROUTER() (common.Address, error) {
	return _Contract.Contract.ICS26ROUTER(&_Contract.CallOpts)
}

// ICS26ROUTER is a free data retrieval call binding the contract method 0xc20f00e1.
//
// Solidity: function ICS26_ROUTER() view returns(address)
func (_Contract *ContractCallerSession) ICS26ROUTER() (common.Address, error) {
	return _Contract.Contract.ICS26ROUTER(&_Contract.CallOpts)
}

// GetAttestationSet is a free data retrieval call binding the contract method 0xa845a7f3.
//
// Solidity: function getAttestationSet() view returns(address[] attestorAddresses, uint8 minRequiredSigs)
func (_Contract *ContractCaller) GetAttestationSet(opts *bind.CallOpts) (struct {
	AttestorAddresses []common.Address
	MinRequiredSigs   uint8
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getAttestationSet")

	outstruct := new(struct {
		AttestorAddresses []common.Address
		MinRequiredSigs   uint8
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.AttestorAddresses = *abi.ConvertType(out[0], new([]common.Address)).(*[]common.Address)
	outstruct.MinRequiredSigs = *abi.ConvertType(out[1], new(uint8)).(*uint8)

	return *outstruct, err

}

// GetAttestationSet is a free data retrieval call binding the contract method 0xa845a7f3.
//
// Solidity: function getAttestationSet() view returns(address[] attestorAddresses, uint8 minRequiredSigs)
func (_Contract *ContractSession) GetAttestationSet() (struct {
	AttestorAddresses []common.Address
	MinRequiredSigs   uint8
}, error) {
	return _Contract.Contract.GetAttestationSet(&_Contract.CallOpts)
}

// GetAttestationSet is a free data retrieval call binding the contract method 0xa845a7f3.
//
// Solidity: function getAttestationSet() view returns(address[] attestorAddresses, uint8 minRequiredSigs)
func (_Contract *ContractCallerSession) GetAttestationSet() (struct {
	AttestorAddresses []common.Address
	MinRequiredSigs   uint8
}, error) {
	return _Contract.Contract.GetAttestationSet(&_Contract.CallOpts)
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

// GetConsensusTimestamp is a free data retrieval call binding the contract method 0x5832611b.
//
// Solidity: function getConsensusTimestamp(uint64 revisionHeight) view returns(uint64)
func (_Contract *ContractCaller) GetConsensusTimestamp(opts *bind.CallOpts, revisionHeight uint64) (uint64, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getConsensusTimestamp", revisionHeight)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// GetConsensusTimestamp is a free data retrieval call binding the contract method 0x5832611b.
//
// Solidity: function getConsensusTimestamp(uint64 revisionHeight) view returns(uint64)
func (_Contract *ContractSession) GetConsensusTimestamp(revisionHeight uint64) (uint64, error) {
	return _Contract.Contract.GetConsensusTimestamp(&_Contract.CallOpts, revisionHeight)
}

// GetConsensusTimestamp is a free data retrieval call binding the contract method 0x5832611b.
//
// Solidity: function getConsensusTimestamp(uint64 revisionHeight) view returns(uint64)
func (_Contract *ContractCallerSession) GetConsensusTimestamp(revisionHeight uint64) (uint64, error) {
	return _Contract.Contract.GetConsensusTimestamp(&_Contract.CallOpts, revisionHeight)
}

// Misbehaviour is a free data retrieval call binding the contract method 0xddba6537.
//
// Solidity: function misbehaviour(bytes ) view returns()
func (_Contract *ContractCaller) Misbehaviour(opts *bind.CallOpts, arg0 []byte) error {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "misbehaviour", arg0)

	if err != nil {
		return err
	}

	return err

}

// Misbehaviour is a free data retrieval call binding the contract method 0xddba6537.
//
// Solidity: function misbehaviour(bytes ) view returns()
func (_Contract *ContractSession) Misbehaviour(arg0 []byte) error {
	return _Contract.Contract.Misbehaviour(&_Contract.CallOpts, arg0)
}

// Misbehaviour is a free data retrieval call binding the contract method 0xddba6537.
//
// Solidity: function misbehaviour(bytes ) view returns()
func (_Contract *ContractCallerSession) Misbehaviour(arg0 []byte) error {
	return _Contract.Contract.Misbehaviour(&_Contract.CallOpts, arg0)
}

// Paused is a free data retrieval call binding the contract method 0x5c975abb.
//
// Solidity: function paused() view returns(bool)
func (_Contract *ContractCaller) Paused(opts *bind.CallOpts) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "paused")

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// Paused is a free data retrieval call binding the contract method 0x5c975abb.
//
// Solidity: function paused() view returns(bool)
func (_Contract *ContractSession) Paused() (bool, error) {
	return _Contract.Contract.Paused(&_Contract.CallOpts)
}

// Paused is a free data retrieval call binding the contract method 0x5c975abb.
//
// Solidity: function paused() view returns(bool)
func (_Contract *ContractCallerSession) Paused() (bool, error) {
	return _Contract.Contract.Paused(&_Contract.CallOpts)
}

// VerifyMembership is a free data retrieval call binding the contract method 0x682ed5f0.
//
// Solidity: function verifyMembership((bytes,(uint64,uint64),bytes[],bytes) msg_) view returns(uint256)
func (_Contract *ContractCaller) VerifyMembership(opts *bind.CallOpts, msg_ ILightClientMsgsMsgVerifyMembership) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "verifyMembership", msg_)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// VerifyMembership is a free data retrieval call binding the contract method 0x682ed5f0.
//
// Solidity: function verifyMembership((bytes,(uint64,uint64),bytes[],bytes) msg_) view returns(uint256)
func (_Contract *ContractSession) VerifyMembership(msg_ ILightClientMsgsMsgVerifyMembership) (*big.Int, error) {
	return _Contract.Contract.VerifyMembership(&_Contract.CallOpts, msg_)
}

// VerifyMembership is a free data retrieval call binding the contract method 0x682ed5f0.
//
// Solidity: function verifyMembership((bytes,(uint64,uint64),bytes[],bytes) msg_) view returns(uint256)
func (_Contract *ContractCallerSession) VerifyMembership(msg_ ILightClientMsgsMsgVerifyMembership) (*big.Int, error) {
	return _Contract.Contract.VerifyMembership(&_Contract.CallOpts, msg_)
}

// VerifyNonMembership is a free data retrieval call binding the contract method 0x4d6d9ffb.
//
// Solidity: function verifyNonMembership((bytes,(uint64,uint64),bytes[]) msg_) view returns(uint256)
func (_Contract *ContractCaller) VerifyNonMembership(opts *bind.CallOpts, msg_ ILightClientMsgsMsgVerifyNonMembership) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "verifyNonMembership", msg_)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// VerifyNonMembership is a free data retrieval call binding the contract method 0x4d6d9ffb.
//
// Solidity: function verifyNonMembership((bytes,(uint64,uint64),bytes[]) msg_) view returns(uint256)
func (_Contract *ContractSession) VerifyNonMembership(msg_ ILightClientMsgsMsgVerifyNonMembership) (*big.Int, error) {
	return _Contract.Contract.VerifyNonMembership(&_Contract.CallOpts, msg_)
}

// VerifyNonMembership is a free data retrieval call binding the contract method 0x4d6d9ffb.
//
// Solidity: function verifyNonMembership((bytes,(uint64,uint64),bytes[]) msg_) view returns(uint256)
func (_Contract *ContractCallerSession) VerifyNonMembership(msg_ ILightClientMsgsMsgVerifyNonMembership) (*big.Int, error) {
	return _Contract.Contract.VerifyNonMembership(&_Contract.CallOpts, msg_)
}

// Pause is a paid mutator transaction binding the contract method 0x8456cb59.
//
// Solidity: function pause() returns()
func (_Contract *ContractTransactor) Pause(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "pause")
}

// Pause is a paid mutator transaction binding the contract method 0x8456cb59.
//
// Solidity: function pause() returns()
func (_Contract *ContractSession) Pause() (*types.Transaction, error) {
	return _Contract.Contract.Pause(&_Contract.TransactOpts)
}

// Pause is a paid mutator transaction binding the contract method 0x8456cb59.
//
// Solidity: function pause() returns()
func (_Contract *ContractTransactorSession) Pause() (*types.Transaction, error) {
	return _Contract.Contract.Pause(&_Contract.TransactOpts)
}

// Unpause is a paid mutator transaction binding the contract method 0x3f4ba83a.
//
// Solidity: function unpause() returns()
func (_Contract *ContractTransactor) Unpause(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "unpause")
}

// Unpause is a paid mutator transaction binding the contract method 0x3f4ba83a.
//
// Solidity: function unpause() returns()
func (_Contract *ContractSession) Unpause() (*types.Transaction, error) {
	return _Contract.Contract.Unpause(&_Contract.TransactOpts)
}

// Unpause is a paid mutator transaction binding the contract method 0x3f4ba83a.
//
// Solidity: function unpause() returns()
func (_Contract *ContractTransactorSession) Unpause() (*types.Transaction, error) {
	return _Contract.Contract.Unpause(&_Contract.TransactOpts)
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

// ContractPausedIterator is returned from FilterPaused and is used to iterate over the raw logs and unpacked data for Paused events raised by the Contract contract.
type ContractPausedIterator struct {
	Event *ContractPaused // Event containing the contract specifics and raw log

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
func (it *ContractPausedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractPaused)
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
		it.Event = new(ContractPaused)
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
func (it *ContractPausedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractPausedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractPaused represents a Paused event raised by the Contract contract.
type ContractPaused struct {
	Account common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterPaused is a free log retrieval operation binding the contract event 0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258.
//
// Solidity: event Paused(address account)
func (_Contract *ContractFilterer) FilterPaused(opts *bind.FilterOpts) (*ContractPausedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Paused")
	if err != nil {
		return nil, err
	}
	return &ContractPausedIterator{contract: _Contract.contract, event: "Paused", logs: logs, sub: sub}, nil
}

// WatchPaused is a free log subscription operation binding the contract event 0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258.
//
// Solidity: event Paused(address account)
func (_Contract *ContractFilterer) WatchPaused(opts *bind.WatchOpts, sink chan<- *ContractPaused) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Paused")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractPaused)
				if err := _Contract.contract.UnpackLog(event, "Paused", log); err != nil {
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

// ParsePaused is a log parse operation binding the contract event 0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258.
//
// Solidity: event Paused(address account)
func (_Contract *ContractFilterer) ParsePaused(log types.Log) (*ContractPaused, error) {
	event := new(ContractPaused)
	if err := _Contract.contract.UnpackLog(event, "Paused", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractUnpausedIterator is returned from FilterUnpaused and is used to iterate over the raw logs and unpacked data for Unpaused events raised by the Contract contract.
type ContractUnpausedIterator struct {
	Event *ContractUnpaused // Event containing the contract specifics and raw log

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
func (it *ContractUnpausedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractUnpaused)
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
		it.Event = new(ContractUnpaused)
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
func (it *ContractUnpausedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractUnpausedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractUnpaused represents a Unpaused event raised by the Contract contract.
type ContractUnpaused struct {
	Account common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterUnpaused is a free log retrieval operation binding the contract event 0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa.
//
// Solidity: event Unpaused(address account)
func (_Contract *ContractFilterer) FilterUnpaused(opts *bind.FilterOpts) (*ContractUnpausedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Unpaused")
	if err != nil {
		return nil, err
	}
	return &ContractUnpausedIterator{contract: _Contract.contract, event: "Unpaused", logs: logs, sub: sub}, nil
}

// WatchUnpaused is a free log subscription operation binding the contract event 0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa.
//
// Solidity: event Unpaused(address account)
func (_Contract *ContractFilterer) WatchUnpaused(opts *bind.WatchOpts, sink chan<- *ContractUnpaused) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Unpaused")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractUnpaused)
				if err := _Contract.contract.UnpackLog(event, "Unpaused", log); err != nil {
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

// ParseUnpaused is a log parse operation binding the contract event 0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa.
//
// Solidity: event Unpaused(address account)
func (_Contract *ContractFilterer) ParseUnpaused(log types.Log) (*ContractUnpaused, error) {
	event := new(ContractUnpaused)
	if err := _Contract.contract.UnpackLog(event, "Unpaused", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
