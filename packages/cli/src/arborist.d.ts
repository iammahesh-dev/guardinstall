declare module '@npmcli/arborist' {
  export default class Arborist {
    constructor(options: { path: string })
    loadActual(): Promise<Node>
    loadVirtual(): Promise<Node>
  }

  export interface Node {
    inventory: Map<string, Node>
    name: string
    version: string
    path: string
    package?: {
      scripts?: Record<string, string>
    }
  }
}
