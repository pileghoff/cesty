#include "llvm/IR/DataLayout.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/Passes/PassBuilder.h"
#if __has_include("llvm/Plugins/PassPlugin.h")
#include "llvm/Plugins/PassPlugin.h"
#else
#include "llvm/Passes/PassPlugin.h"
#endif
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
  FunctionCallee MemMoveFunc;
  FunctionCallee MemSetFunc;
  FunctionCallee MemCmpFunc;
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

    MemMoveFunc = F.getParent()->getOrInsertFunction(
        "cesty_memmove",
        FunctionType::get(Type::getVoidTy(F.getParent()->getContext()),
                          {EntryBuilder.getPtrTy(), EntryBuilder.getPtrTy(),
                           EntryBuilder.getInt64Ty()},
                          false));

    MemSetFunc = F.getParent()->getOrInsertFunction(
        "cesty_memset",
        FunctionType::get(Type::getVoidTy(F.getParent()->getContext()),
                          {EntryBuilder.getPtrTy(), EntryBuilder.getInt64Ty(),
                           EntryBuilder.getInt64Ty()},
                          false));

    MemCmpFunc = F.getParent()->getOrInsertFunction(
        "cesty_memcmp",
        FunctionType::get(EntryBuilder.getInt32Ty(),
                          {EntryBuilder.getPtrTy(), EntryBuilder.getPtrTy(),
                           EntryBuilder.getInt64Ty()},
                          false));
    SmallVector<StoreInst *> Stores;
    SmallVector<LoadInst *> Loads;
    SmallVector<MemCpyInst *> Copies;
    SmallVector<MemSetInst *> Sets;
    SmallVector<MemMoveInst *> Moves;
    SmallVector<CallInst *> Cmps;

    for (BasicBlock &BB : F) {
      for (Instruction &I : BB) {
        if (auto *SI = dyn_cast<StoreInst>(&I)) {
          Stores.push_back(SI);
        }

        if (auto *LI = dyn_cast<LoadInst>(&I)) {
          Loads.push_back(LI);
        }

        if (auto *MI = dyn_cast<MemCpyInst>(&I)) {
          Copies.push_back(MI);
        }

        if (auto *MM = dyn_cast<MemMoveInst>(&I)) {
          Moves.push_back(MM);
        }

        if (auto *MS = dyn_cast<MemSetInst>(&I)) {
          Sets.push_back(MS);
        }

        if (auto *CI = dyn_cast<CallInst>(&I)) {
          auto *Callee = CI->getCalledFunction();
          if (Callee && Callee->getName() == "memcmp") {
            Cmps.push_back(CI);
          }
        }
      }
    }

    for (LoadInst *LI : Loads) {
      handle_load(LI);
    }

    for (StoreInst *SI : Stores) {
      handle_store(SI);
    }

    for (MemCpyInst *MI : Copies) {
      handle_memcpy(MI);
    }

    for (MemSetInst *MS : Sets) {
      handle_memset(MS);
    }

    for (CallInst *CI : Cmps) {
      handle_memcmp(CI);
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

  void handle_memcmp(CallInst *CI) {
    IRBuilder<> Builder(CI);
    CI->replaceAllUsesWith(Builder.CreateCall(
        MemCmpFunc, {CI->getArgOperand(0), CI->getArgOperand(1),

                     CI->getArgOperand(2)}));
    CI->eraseFromParent();
  }

  void handle_memmove(MemMoveInst *MM) {

    IRBuilder<> Builder(MM);
    Builder.CreateCall(MemMoveFunc,
                       {MM->getDest(), MM->getSource(), MM->getLength()});

    MM->eraseFromParent();
  }

  void handle_memcpy(MemCpyInst *MI) {

    IRBuilder<> Builder(MI);
    Builder.CreateCall(MemMoveFunc,
                       {MI->getDest(), MI->getSource(), MI->getLength()});

    MI->eraseFromParent();
  }

  void handle_memset(MemSetInst *MS) {

    IRBuilder<> Builder(MS);
    Builder.CreateCall(MemSetFunc,
                       {MS->getDest(), MS->getValue(), MS->getLength()});

    MS->eraseFromParent();
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
    LoadInst *Replacement =
        Builder.CreateLoad(Ty, Tmp, LI->isVolatile(), "cesty.loaded");

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
    Builder.CreateStore(Val, Tmp, SI->isVolatile());

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
