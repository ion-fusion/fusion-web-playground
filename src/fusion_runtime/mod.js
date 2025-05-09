export async function js_create_classloader() {
    await cheerpjInit();
    return await cheerpjRunLibrary("/app/fusion-java.jar:/app/ion-java.jar");
}

export async function js_create_fusion_runtime(classloader) {
    const FusionRuntimeBuilder = await classloader.dev.ionfusion.fusion.FusionRuntimeBuilder;
    const standardRuntimeBuilder = await FusionRuntimeBuilder.standard();
    return await standardRuntimeBuilder.build();
}

export async function js_fusion_eval(classloader, runtime, expr) {
    const sandboxBuilder = await runtime.makeSandboxBuilder();
    await sandboxBuilder.setLanguage("/fusion");
    const toplevel = await sandboxBuilder.build();

    const FusionIo = await classloader.dev.ionfusion.fusion.FusionIo;

    try {
        const result = await toplevel.eval(expr);
        return await FusionIo.writeToString(toplevel, result);
    } catch (ex) {
        const error = await ex.toString();
        throw error;
    }
}