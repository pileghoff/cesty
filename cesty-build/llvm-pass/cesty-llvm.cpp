#include "llvm/IR/DataLayout.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Support/raw_ostream.h"
#include <tuple>
#include <vector>
using namespace llvm;

namespace {

class CestyFuncHandler {
public:
  CestyFuncHandler() = delete;
  IRBuilder<> EntryBuilder;
  std::vector<std::tuple<Type *, AllocaInst *>> allocs;
  Function &F;
  FunctionCallee LoadFunc;
  FunctionCallee StoreFunc;
  const DataLayout &DL;
  CestyFuncHandler(Function &F)
      : EntryBuilder(&F.getEntryBlock(), F.getEntryBlock().begin()), F(F),
        DL(F.getParent()->getDataLayout()) {

    LoadFunc = F.getParent()->getOrInsertFunction(
        "cesty_load",
        FunctionType::get(Type::getVoidTy(F.getParent()->getContext()),
                          {EntryBuilder.getPtrTy(), EntryBuilder.getPtrTy(),
                           EntryBuilder.getInt64Ty()},
                          false));

    StoreFunc = F.getParent()->getOrInsertFunction(
        "cesty_store",
        FunctionType::get(Type::getVoidTy(F.getParent()->getContext()),
                          {EntryBuilder.getPtrTy(), EntryBuilder.getPtrTy(),
                           EntryBuilder.getInt64Ty()},
                          false));
    SmallVector<StoreInst *> Stores;
    SmallVector<LoadInst *> Loads;

    for (BasicBlock &BB : F) {
      for (Instruction &I : BB) {
        if (auto *SI = dyn_cast<StoreInst>(&I)) {
          Stores.push_back(SI);
        }

        if (auto *LI = dyn_cast<LoadInst>(&I)) {
          Loads.push_back(LI);
        }
      }
    }

    for (LoadInst *LI : Loads) {
      handle_load(LI);
    }

    for (StoreInst *SI : Stores) {
      handle_store(SI);
    }
  }

  AllocaInst *getAlloca(Type *Ty) {
    for (int i = 0; i < allocs.size(); i++) {
      auto entry = allocs[i];
      if (std::get<0>(entry) == Ty) {
        return std::get<1>(entry);
      }
    }

    AllocaInst *Tmp = EntryBuilder.CreateAlloca(Ty, nullptr, "cesty.tmp");
    allocs.push_back({Ty, Tmp});
    return Tmp;
  }

  void handle_load(LoadInst *LI) {
    Value *Src = LI->getPointerOperand();
    Type *Ty = LI->getType();

    // Insert replacement at original load site
    IRBuilder<> Builder(LI);

    AllocaInst *Tmp = getAlloca(Ty);
    uint64_t Size = DL.getTypeStoreSize(Ty);

    Value *SizeVal =
        ConstantInt::get(Type::getInt64Ty(LI->getModule()->getContext()), Size);

    // Call runtime
    Builder.CreateCall(LoadFunc, {Src, Tmp, SizeVal});

    // Load from temp
    LoadInst *Replacement = Builder.CreateLoad(Ty, Tmp, "cesty.loaded");

    // Redirect uses
    LI->replaceAllUsesWith(Replacement);

    // Remove original load
    LI->eraseFromParent();
  }

  void handle_store(StoreInst *SI) {

    Value *Dst = SI->getPointerOperand();
    Value *Val = SI->getValueOperand();

    Type *Ty = Val->getType();

    // Insert replacement at original store location
    IRBuilder<> Builder(SI);

    // Write value into temp
    AllocaInst *Tmp = getAlloca(Ty);
    Builder.CreateStore(Val, Tmp);

    // Compute byte size
    uint64_t Size = DL.getTypeStoreSize(Ty);
    Value *SizeVal =
        ConstantInt::get(Type::getInt64Ty(SI->getModule()->getContext()), Size);

    // Call StoreFunc
    Builder.CreateCall(StoreFunc, {Dst, Tmp, SizeVal});

    // Remove original store
    SI->eraseFromParent();
  }
};

class CestyPass : public PassInfoMixin<CestyPass> {
public:
  PreservedAnalyses run(Function &F, FunctionAnalysisManager &) {

    CestyFuncHandler handler(F);
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
