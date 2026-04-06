import { Alloy } from "../../runtime";

export function useConfig(schema: any) {
  // @ts-ignore
  const accessor = AlloyInternal.useConfig(Alloy.pluginName, schema);

  return {
    ...accessor,
    set(key: string, value: any, options?: any) {
      // @ts-ignore
      AlloyInternal.configSet(Alloy.pluginName, key, value, options);
    }
  };
}
