#include "llvm/IR/DataLayout.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Support/raw_ostream.h"

using namespace llvm;

namespace {

class CestyPass : public PassInfoMixin<CestyPass> {
public:
  PreservedAnalyses run(Function &F, FunctionAnalysisManager &) {

    Module *M = F.getParent();
    LLVMContext &Ctx = M->getContext();

    SmallVector<StoreInst *> Stores;
    SmallVector<LoadInst *> Loads;

    for (BasicBlock &BB : F) {
      for (Instruction &I : BB) {
        if (auto *SI = dyn_cast<StoreInst>(&I))
          Stores.push_back(SI);

        if (auto *LI = dyn_cast<LoadInst>(&I))
          Loads.push_back(LI);
      }
    }
    for (LoadInst *LI : Loads) {

      Module *M = LI->getModule();
      LLVMContext &Ctx = M->getContext();
      const DataLayout &DL = M->getDataLayout();

      Value *Src = LI->getPointerOperand();
      Type *Ty = LI->getType();

      // Allocate temp storage in entry block
      IRBuilder<> EntryBuilder(&F.getEntryBlock(), F.getEntryBlock().begin());

      AllocaInst *Tmp = EntryBuilder.CreateAlloca(Ty, nullptr, "cesty.tmp");

      // Insert replacement at original load site
      IRBuilder<> Builder(LI);

      uint64_t Size = DL.getTypeStoreSize(Ty);

      Value *SizeVal = ConstantInt::get(Type::getInt64Ty(Ctx), Size);

      // Declare:
      // void cesty_load(ptr src,
      //                 ptr dst,
      //                 i64 size)
      FunctionCallee Hook = M->getOrInsertFunction(
          "cesty_load",
          FunctionType::get(
              Type::getVoidTy(Ctx),
              {Builder.getPtrTy(), Builder.getPtrTy(), Builder.getInt64Ty()},
              false));

      // Call runtime
      Builder.CreateCall(Hook, {Src, Tmp, SizeVal});

      // Load from temp
      LoadInst *Replacement = Builder.CreateLoad(Ty, Tmp, "cesty.loaded");

      // Redirect uses
      LI->replaceAllUsesWith(Replacement);

      // Remove original load
      LI->eraseFromParent();
    }

    for (StoreInst *SI : Stores) {

      Module *M = SI->getModule();
      LLVMContext &Ctx = M->getContext();
      const DataLayout &DL = M->getDataLayout();

      Value *Dst = SI->getPointerOperand();
      Value *Val = SI->getValueOperand();

      Type *Ty = Val->getType();

      // Create temp storage in function entry block
      IRBuilder<> EntryBuilder(&F.getEntryBlock(), F.getEntryBlock().begin());

      AllocaInst *Tmp = EntryBuilder.CreateAlloca(Ty, nullptr, "cesty.tmp");

      // Insert replacement at original store location
      IRBuilder<> Builder(SI);

      // Write value into temp
      Builder.CreateStore(Val, Tmp);

      // Compute byte size
      uint64_t Size = DL.getTypeStoreSize(Ty);

      Value *SizeVal = ConstantInt::get(Type::getInt64Ty(Ctx), Size);

      // Declare:
      // void cesty_store(ptr dst,
      //                  ptr src,
      //                  i64 size)
      FunctionCallee Hook = M->getOrInsertFunction(
          "cesty_store",
          FunctionType::get(
              Type::getVoidTy(Ctx),
              {Builder.getPtrTy(), Builder.getPtrTy(), Builder.getInt64Ty()},
              false));

      // Call runtime
      Builder.CreateCall(Hook, {Dst, Tmp, SizeVal});

      // Remove original store
      SI->eraseFromParent();
    }

    return PreservedAnalyses::none();
  }

  static bool isRequired() { return true; }
};
} // namespace

extern "C" LLVM_ATTRIBUTE_WEAK llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
  return {LLVM_PLUGIN_API_VERSION, "cesty", "0.1", [](llvm::PassBuilder &PB) {
            PB.registerPipelineStartEPCallback([](ModulePassManager &MPM,
                                                  OptimizationLevel Level) {
              FunctionPassManager FPM;
              FPM.addPass(CestyPass());

              MPM.addPass(createModuleToFunctionPassAdaptor(std::move(FPM)));
            });
          }};
}
