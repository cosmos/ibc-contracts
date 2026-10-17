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
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"attestorAddresses\",\"type\":\"address[]\",\"internalType\":\"address[]\"},{\"name\":\"minRequiredSigs\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"initialHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"initialTimestampSeconds\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"authority\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getAttestationSet\",\"inputs\":[],\"outputs\":[{\"name\":\"attestorAddresses\",\"type\":\"address[]\",\"internalType\":\"address[]\"},{\"name\":\"minRequiredSigs\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getClientState\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getConsensusTimestamp\",\"inputs\":[{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isConsumingScheduledOp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"misbehaviour\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setAuthority\",\"inputs\":[{\"name\":\"newAuthority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"updateMsg\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"enumILightClientMsgs.UpdateResult\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"verifyMembership\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"verifyNonMembership\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structILightClientMsgs.MsgVerifyNonMembership\",\"components\":[{\"name\":\"proof\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"proofHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"path\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"AuthorityUpdated\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AccessManagedInvalidAuthority\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AccessManagedRequiredDelay\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"delay\",\"type\":\"uint32\",\"internalType\":\"uint32\"}]},{\"type\":\"error\",\"name\":\"AccessManagedUnauthorized\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"BadQuorum\",\"inputs\":[{\"name\":\"minRequired\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"attestationCount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ConsensusTimestampNotFound\",\"inputs\":[{\"name\":\"height\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"DuplicateSigner\",\"inputs\":[{\"name\":\"signer\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignature\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignatureLength\",\"inputs\":[{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignatureS\",\"inputs\":[{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"EmptyPackets\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EmptySignatures\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EmptyValue\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FeatureNotSupported\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FrozenClientState\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"HeightMismatch\",\"inputs\":[{\"name\":\"expected\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"provided\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"InvalidPathLength\",\"inputs\":[{\"name\":\"expectedLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"providedLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidSignatureLength\",\"inputs\":[{\"name\":\"signature\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"InvalidState\",\"inputs\":[{\"name\":\"height\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"NoAttestors\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotMember\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotNonMember\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SignatureInvalid\",\"inputs\":[{\"name\":\"signature\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"ThresholdNotMet\",\"inputs\":[{\"name\":\"validSigners\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"minRequired\",\"type\":\"uint8\",\"internalType\":\"uint8\"}]},{\"type\":\"error\",\"name\":\"UnknownSigner\",\"inputs\":[{\"name\":\"signer\",\"type\":\"address\",\"internalType\":\"address\"}]}]",
	Bin: "0x60806040523461034957611a088038038061001981610365565b928339810160a0828203126103495781516001600160401b0381116103495782019181601f84011215610349578251916001600160401b0383116102d2578260051b936020610069818701610365565b8095815201906020829682010192831161034957602001905b82821061034d5750505060208101519260ff8416809403610349576100a96040830161039e565b906100c260806100bb6060860161039e565b940161038a565b5f80546001600160a01b0319166001600160a01b0383169081179091556040518181529091907f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad90602090a13b15610337575083511561032857841515806102fc575b845190156102e6575060405194608086016001600160401b038111878210176102d257604052848652602086019081526060604087019360018060401b03169687855201915f835285519060018060401b0382116102d2576801000000000000000082116102d257600154826001558083106102a7575b5060015f5260205f205f5b83811061028a575050505060ff90511669ff00000000000000000060025493610100600160481b03905160081b169251151560481b169260018060501b0319161717176002555f5b825181101561024f5760018060a01b0360208260051b8501015116805f52600360205260ff60405f20541661023d57906001915f52600360205260405f208260ff19825416179055016101ef565b637010e27960e11b5f5260045260245ffd5b505f838152600460205260409081902080546001600160401b0319166001600160401b03909316929092179091555161165590816103b38239f35b82516001600160a01b0316818301556020909201916001016101a7565b60015f52828060205f20019103905f5b8281106102c557505061019c565b5f828201556001016102b7565b634e487b7160e01b5f52604160045260245ffd5b856365846b7f60e01b5f5260045260245260445ffd5b5083515f19860160ff81116103145760ff1610610125565b634e487b7160e01b5f52601160045260245ffd5b6343d0a89360e11b5f5260045ffd5b6361798f2f60e11b5f5260045260245ffd5b5f80fd5b6020809161035a8461038a565b815201910190610082565b6040519190601f01601f191682016001600160401b038111838210176102d257604052565b51906001600160a01b038216820361034957565b51906001600160401b03821682036103495756fe60806040526004361015610011575f80fd5b5f3560e01c80630bece356146104af5780634d6d9ffb146104595780635832611b14610413578063682ed5f0146103b55780637a9e5e4b146102b05780638fb360371461023d578063a845a7f3146101b0578063bf7e214f1461017e578063ddba6537146101075763ef913a4b14610087575f80fd5b34610103575f600319360112610103576100ff604051602080820152608060408201526100f3816100ba60c08201610f6f565b60ff600254818116606085015267ffffffffffffffff8160081c16608085015260481c16151560a083015203601f1981018352826105e1565b6040519182918261056e565b0390f35b5f80fd5b34610103576101153661051d565b505060ff60025460481c166101565761012e3633610fbe565b7fda81d7c2000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f928b1233000000000000000000000000000000000000000000000000000000005f5260045ffd5b34610103575f60031936011261010357602073ffffffffffffffffffffffffffffffffffffffff5f5416604051908152f35b34610103575f6003193601126101035760ff600254166040516101dd816101d681610f6f565b03826105e1565b60405190604082019260408352815180945260206060840192015f945b80861061020f57505082935060208301520390f35b909260208060019273ffffffffffffffffffffffffffffffffffffffff8751168152019401950194906101fa565b34610103575f600319360112610103575f5460a01c60ff16156102a85760207f8fb36037000000000000000000000000000000000000000000000000000000005b7fffffffff0000000000000000000000000000000000000000000000000000000060405191168152f35b60205f61027e565b346101035760206003193601126101035760043573ffffffffffffffffffffffffffffffffffffffff811690818103610103575f549073ffffffffffffffffffffffffffffffffffffffff82163303610389573b1561035d57817f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad927fffffffffffffffffffffffff000000000000000000000000000000000000000060209316175f55604051908152a1005b507fc2f31e5e000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7f068ca9d8000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b346101035760206003193601126101035760043567ffffffffffffffff81116101035760a060031982360301126101035760ff60025460481c166101565761040b6020916104033633610fbe565b600401610e12565b604051908152f35b346101035760206003193601126101035760043567ffffffffffffffff8116809103610103575f526004602052602067ffffffffffffffff60405f205416604051908152f35b346101035760206003193601126101035760043567ffffffffffffffff811161010357608060031982360301126101035760ff60025460481c166101565761040b6020916104a73633610fbe565b600401610c3c565b34610103576104bd3661051d565b60ff60025460481c16610156576104dd916104d83633610fbe565b610783565b60405160038210156104f0576020918152f35b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602160045260245ffd5b9060206003198301126101035760043567ffffffffffffffff811161010357826023820112156101035780600401359267ffffffffffffffff84116101035760248483010111610103576024019190565b601f19601f602060409481855280519182918282880152018686015e5f8582860101520116010190565b6040810190811067ffffffffffffffff8211176105b457604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b90601f601f19910116810190811067ffffffffffffffff8211176105b457604052565b92919267ffffffffffffffff82116105b4576040519161062e6020601f19601f84011601846105e1565b829481845281830111610103578281602093845f960137010152565b9080601f830112156101035781602061066593359101610604565b90565b67ffffffffffffffff81116105b45760051b60200190565b6020818303126101035780359067ffffffffffffffff821161010357019060408282031261010357604051916106b583610598565b803567ffffffffffffffff811161010357826106d291830161064a565b835260208101359067ffffffffffffffff821161010357019080601f8301121561010357813561070181610668565b9261070f60405194856105e1565b81845260208085019260051b820101918383116101035760208201905b838210610740575050505050602082015290565b813567ffffffffffffffff8111610103576020916107638784809488010161064a565b81520191019061072c565b519067ffffffffffffffff8216820361010357565b61078f91810190610680565b60205f818351604051918183925191829101835e8101838152039060025afa156109d85760205f8051604051838101917f010000000000000000000000000000000000000000000000000000000000000083526021820152602181526107f66041826105e1565b604051918291518091835e8101838152039060025afa156109d8576108215f51602083015190611251565b516040818051810103126101035760405161083b81610598565b610856604061084c6020850161076e565b938484520161076e565b67ffffffffffffffff602083019382855216801515806109c5575b1561098b57505067ffffffffffffffff8151165f52600460205267ffffffffffffffff60405f2054168061093a575067ffffffffffffffff9051918183169260025490838260081c1685116108fd575b50505116905f52600460205260405f20907fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000008254161790555f90565b68ffffffffffffffff007fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000ff9160081b169116176002555f806108c1565b9167ffffffffffffffff915051160361095257600290565b69010000000000000000007fffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffffff6002541617600255600190565b67ffffffffffffffff92507fdcbc5460000000000000000000000000000000000000000000000000000000005f526004521660245260445ffd5b5067ffffffffffffffff82161515610871565b6040513d5f823e3d90fd5b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe181360301821215610103570180359067ffffffffffffffff821161010357602001918160051b3603831361010357565b3567ffffffffffffffff811681036101035790565b15610a545750565b67ffffffffffffffff907ff7caaa5c000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe181360301821215610103570180359067ffffffffffffffff82116101035760200191813603831361010357565b6020818303126101035780519067ffffffffffffffff82116101035701906040828203126101035760405191610b1083610598565b610b198161076e565b835260208101519067ffffffffffffffff8211610103570181601f8201121561010357805190610b4882610668565b92610b5660405194856105e1565b82845260208085019360061b8301019181831161010357602001925b828410610b855750505050602082015290565b6040848303126101035760206040918251610b9f81610598565b865181528287015183820152815201930192610b72565b15610bbf575050565b9067ffffffffffffffff80927fc7441689000000000000000000000000000000000000000000000000000000005f52166004521660245260445ffd5b8051821015610c0f5760209160051b010190565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b6060810190610c4b82826109e3565b90506001610c5984846109e3565b92905003610de25750610cff610c7160408301610a37565b9267ffffffffffffffff841692835f526004602052610cfa67ffffffffffffffff60405f20541694610ca587871515610a4c565b610ce5610cbd610cb58580610a8a565b810190610680565b610cd5610cca82516111cb565b602083015190611251565b5160208082518301019101610adb565b9667ffffffffffffffff885116918214610bb6565b6109e3565b15610c0f57610d1a610d1382602093610a8a565b3691610604565b818151910120920180515115610dba575f5b81518051821015610d9257610d42828692610bfb565b515114610d5157600101610d2c565b6020929350610d609151610bfb565b510151610d6a5790565b7f9a960b4f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f291fc442000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f1e6c84da000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f88b3170e000000000000000000000000000000000000000000000000000000005f52600160045260245260445ffd5b60808101610e208183610a8a565b905015610f47576060820190610e3682846109e3565b90506001610e4484866109e3565b92905003610de25750610e5960408401610a37565b90610ebb67ffffffffffffffff831693845f526004602052610eb567ffffffffffffffff60405f20541695610e9086881515610a4c565b610ea0610cbd610cb58a80610a8a565b9567ffffffffffffffff875116918214610bb6565b856109e3565b15610c0f57602091610ed3610d1383610edf94610a8a565b83815191012095610a8a565b9080929181010312610103576020903591019081515115610dba575f5b82518051821015610d9257610f12828792610bfb565b51511480610f2f575b610f2757600101610efc565b505050905090565b50816020610f3e838651610bfb565b51015114610f1b565b7f1208b21b000000000000000000000000000000000000000000000000000000005f5260045ffd5b602060015491828152019060015f5260205f20905f5b818110610f925750505090565b825473ffffffffffffffffffffffffffffffffffffffff16845260209093019260019283019201610f85565b5f915f549173ffffffffffffffffffffffffffffffffffffffff83169282600411610103575f5f9060405f81519573ffffffffffffffffffffffffffffffffffffffff60208801917fb700961300000000000000000000000000000000000000000000000000000000835216968760248201523060448201527fffffffff0000000000000000000000000000000000000000000000000000000083351660648201526064815261106f6084826105e1565b828052826020525190895afa6111b8575b1561108e575b505050505050565b63ffffffff161561118c577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000175f55823b1561010357601f19925f92836064601f94604051978896879586937f94c7d7ee00000000000000000000000000000000000000000000000000000000855260048501526040602485015280604485015280868686013785858286010152011681010301925af180156109d857611179575b507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff81541690555f8080808080611086565b61118591505f906105e1565b5f5f611147565b507f068ca9d8000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b50505f516020518060201c150290611080565b5f60208092604051918183925191829101835e8101838152039060025afa156109d85760205f8051604051838101917f020000000000000000000000000000000000000000000000000000000000000083526021820152602181526112316041826105e1565b604051918291518091835e8101838152039060025afa156109d8575f5190565b9190918251156114ac57825160ff60025416907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82019060ff821161147f5760ff865192161015611451575050825192601f196112c66112b086610668565b956112be60405197886105e1565b808752610668565b013660208601375f5b815181101561144a576112e28183610bfb565b5160418151036114145773ffffffffffffffffffffffffffffffffffffffff61131761130e83876114d4565b9092919261150e565b169081156113da5750805f52600360205260ff60405f205416156113af575f5b82811061135457509060019161134d8288610bfb565b52016112cf565b8173ffffffffffffffffffffffffffffffffffffffff611374838a610bfb565b51161461138357600101611337565b507fe021c4f2000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7fd7e8a2c2000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b611410906040519182917f9e3c1b3d0000000000000000000000000000000000000000000000000000000083526004830161056e565b0390fd5b611410906040519182917f2ee17a3d0000000000000000000000000000000000000000000000000000000083526004830161056e565b5050509050565b7f378b0805000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b7f301b38b6000000000000000000000000000000000000000000000000000000005f5260045ffd5b8151919060418303611504576114fd9250602082015190606060408401519301515f1a906115b9565b9192909190565b50505f9160029190565b60048110156104f05780611520575050565b60018103611550577ff645eedf000000000000000000000000000000000000000000000000000000005f5260045ffd5b6002810361158457507ffce698f7000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b60031461158e5750565b7fd78bce0c000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b91907f7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0841161163d579160209360809260ff5f9560405194855216868401526040830152606082015282805260015afa156109d8575f5173ffffffffffffffffffffffffffffffffffffffff81161561163357905f905f90565b505f906001905f90565b5050505f916003919056fea164736f6c6343000825000a",
}

// ContractABI is the input ABI used to generate the binding from.
// Deprecated: Use ContractMetaData.ABI instead.
var ContractABI = ContractMetaData.ABI

// ContractBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ContractMetaData.Bin instead.
var ContractBin = ContractMetaData.Bin

// DeployContract deploys a new Ethereum contract, binding an instance of Contract to it.
func DeployContract(auth *bind.TransactOpts, backend bind.ContractBackend, attestorAddresses []common.Address, minRequiredSigs uint8, initialHeight uint64, initialTimestampSeconds uint64, authority common.Address) (common.Address, *types.Transaction, *Contract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ContractBin), backend, attestorAddresses, minRequiredSigs, initialHeight, initialTimestampSeconds, authority)
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
