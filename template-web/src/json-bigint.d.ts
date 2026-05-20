declare module 'json-bigint' {
  interface JSONBigintOptions {
    storeAsString?: boolean
    alwaysParseAsBig?: boolean
    useNativeBigInt?: boolean
    protoAction?: 'error' | 'ignore' | 'preserve'
    constructorAction?: 'error' | 'ignore' | 'preserve'
  }

  interface JSONBigint {
    parse(text: string, reviver?: (key: string, value: any) => any): any
    stringify(value: any, replacer?: ((key: string, value: any) => any) | (string | number)[], space?: string | number): string
  }

  function JSONbig(options?: JSONBigintOptions): JSONBigint

  export default JSONbig
}
