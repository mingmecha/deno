import { splitSelectors } from "./40_lint_selector.js";
import {
  compileSelector,
  MatchCtx,
  MatcherFn,
  parseSelector,
} from "./40_lint_selector.js";
import { expect } from "@std/expect";

/**
 * TS eslint selector Examples
 *
 * ```js
 * ForOfStatement[await=true]
 * VariableDeclaration[kind="await using"]
 * MethodDefinition[kind="constructor"] ThisExpression
 * PropertyDefinition > ArrowFunctionExpression.value
 * PropertyDefinition > *.key
 * ThisExpression, Super
 * VariableDeclarator,PropertyDefinition,:matches(FunctionDeclaration,FunctionExpression) > AssignmentPattern
 * ImportDeclaration[importKind = "type"]
 * ImportSpecifier[importKind = "type"]
 * ExportNamedDeclaration:not([source])
 * UnaryExpression[operator="delete"]
 * AssignmentExpression[operator = "+="], BinaryExpression[operator = "+"]
 * CallExpression > MemberExpression.callee > Identifier[name = "join"].property
 * CallExpression > MemberExpression.callee > Identifier[name = /^(toLocaleString|toString)$/].property
 * ImportDeclaration[importKind!="type"]
 * UnaryExpression[operator="void"]
 * UnaryExpression[operator="!"]
 * LogicalExpression[operator = "??"] > TSNonNullExpression.left
 * CallExpression[callee.name="require"]
 * :not(ArrowFunctionExpression) > TSTypeParameterDeclaration > TSTypeParameter[constraint]
 * ArrowFunctionExpression > TSTypeParameterDeclaration > TSTypeParameter[constraint]
 * PropertyDefinition[value != null]
 * :not(ObjectPattern) > Property
 * CallExpression > *.callee
 * TaggedTemplateExpression > *.tag
 * BinaryExpression[operator=/^[<>!=]?={0,2}$/]
 * :matches(ClassDeclaration, ClassExpression)
 * MethodDefinition[kind="constructor"]
 * MemberExpression[computed=true]
 * TSTypeLiteral[members.length = 1]
 * CallExpression[arguments.length=1] > MemberExpression.callee[property.name="test"][computed=false]
 * CallExpression > MemberExpression.callee
 * CallExpression[arguments.length=1] > MemberExpression
 * CallExpression > MemberExpression.callee[property.name="test"][computed=false]
 * :matches(MethodDefinition, TSMethodSignature)[kind=get]
 * ArrowFunctionExpression[async = true] > :not(BlockStatement, AwaitExpression)
 * ```
 */

const AstNodes: Record<string, number> = {
  Foo: 1,
  Bar: 2,
  Baz: 3,
  Foobar: 4,
};

const AstAttrs: Record<string, number> = {
  _empty: 0,
  key: 1,
  value: 2,
  attr: 3,
  attr2: 4,
  children: 5,
  msg: 6,
};
const toElem = (value: string) => AstNodes[value];
const toAttr = (value: string) => AstAttrs[value];

export interface TestNode {
  type: keyof typeof AstNodes;
  children?: TestNode[];
  [key: string]: number | boolean | string | TestNode | TestNode[] | undefined;
}

export interface FakeProp {
  propId: number;
  name: string;
  value: unknown;
}

export interface FakeNode {
  type: number;
  name: string;
  parentId: number;
  props: FakeProp[];
  original: TestNode;
}

function isFakeNode(x: unknown): x is FakeNode {
  return x !== null && typeof x === "object" && "type" in x &&
    typeof x.type === "number" &&
    "original" in x;
}

class FakeContext implements MatchCtx {
  ids = new Map<number, FakeNode>();
  idByNode = new Map<FakeNode, number>();
  id = 0;

  getAttrPathValue(id: number, props: number[]) {
    const node = this.ids.get(id);
    if (node === undefined) return undefined;

    // deno-lint-ignore no-explicit-any
    let tmp: any = node;
    for (let i = 0; i < props.length; i++) {
      const prop = props[i];

      if (isFakeNode(tmp)) {
        const found = tmp.props.find((node) => node.propId === prop);
        if (!found) return undefined;
        tmp = found.value;
        continue;
      }

      const name = AstAttrs[prop];
      if (!(name in tmp)) return undefined;
      tmp = tmp[name];
    }

    return tmp;
  }

  hasAttrPath(id: number, propId: number[]): boolean {
    return this.getAttrPathValue(id, propId) !== undefined;
  }

  getType(id: number): number {
    const node = this.ids.get(id);
    if (node === undefined) return -1;
    return node.type;
  }

  getFirstChild(id: number): number {
    const node = this.ids.get(id);
    if (node === undefined) return -1;

    let first = -1;
    // First check if there is an array prop
    for (const prop of node.props) {
      if (Array.isArray(prop.value)) {
        if (prop.value.length === 0) return -1;
        return prop.value[0];
      } else if (
        first === -1 && prop.value !== null && typeof prop.value === "object"
      ) {
        first = prop.value;
      }
    }
    return first;
  }

  getLastChild(id: number): number {
    const node = this.ids.get(id);
    if (node === undefined) return -1;

    let last = -1;
    // First check if there is an array prop
    for (const prop of node.props) {
      if (Array.isArray(prop.value)) {
        if (prop.value.length === 0) return -1;
        return prop.value.at(-1);
      } else if (prop.value !== null && typeof prop.value !== "object") {
        last = prop.value;
      }
    }
    return last;
  }

  getParent(id: number): number {
    const node = this.ids.get(id);
    if (node === undefined) return -1;
    return node.parentId;
  }

  getSiblingBefore(parentId: number, sib: number): number {
    const node = this.ids.get(parentId);
    if (node === undefined) return -1;

    let prev = -1;
    for (const prop of node.props) {
      if (prop.value === sib) return prev;

      if (Array.isArray(prop.value)) {
        for (const id of prop.value) {
          if (id === sib) return prev;
          prev = id;
        }
      } else {
        prev = prop.value;
      }
    }

    return -1;
  }

  getSiblings(id: number): number[] {
    const parent = this.getParent(id);
    const node = this.ids.get(parent);
    if (node === undefined) return [];

    for (const prop of node.props) {
      if (Array.isArray(prop.value)) {
        if (prop.value.includes(id)) {
          return prop.value;
        }
      }
    }

    return [];
  }
}

function fakeSerializeAst(node: TestNode): FakeContext {
  const ctx = new FakeContext();
  serializeFakeNode(ctx, node, -1);
  return ctx;
}

function serializeFakeNode(
  ctx: FakeContext,
  node: TestNode,
  parentId: number,
): number {
  const id = ctx.id;
  ctx.id++;

  const type = AstNodes[node.type];

  const props: FakeProp[] = [];
  const fake: FakeNode = {
    type,
    name: node.type,
    parentId,
    props,
    get original() {
      return node;
    },
  };

  ctx.ids.set(id, fake);
  ctx.idByNode.set(fake, id);

  for (const [k, value] of Object.entries(node)) {
    if (k === "type") continue;
    // deno-lint-ignore no-explicit-any
    const propId = (AstAttrs as any)[k] as number;

    const prop: FakeProp = {
      propId: propId,
      name: k,
      value: null,
    };
    props.push(prop);

    if (value !== null && typeof value === "object") {
      if (Array.isArray(value)) {
        prop.value = value.map((v) => serializeFakeNode(ctx, v, id));
      } else {
        prop.value = serializeFakeNode(ctx, value, id);
      }
    } else {
      prop.value = value;
    }
  }

  return id;
}

function visit(
  ctx: FakeContext,
  selector: MatcherFn,
  id: number,
): unknown {
  const node = ctx.ids.get(id)!;
  // console.log("visit", { node });
  const res = selector(ctx, id);
  if (res) {
    // console.log("<-- MATCHED");
    return node.original;
  }

  for (let i = 0; i < node.props.length; i++) {
    const prop = node.props[i];
    const value = prop.value;

    if (Array.isArray(value)) {
      for (let i = 0; i < value.length; i++) {
        const res = visit(ctx, selector, value[i]);
        if (res) {
          return res;
        }
      }
    } else if (isFakeNode(value)) {
      const id = ctx.idByNode.get(value)!;
      const res = visit(ctx, selector, id);
      if (res) {
        return res;
      }
    }
  }
}

function testSelector(
  ast: TestNode,
  selector: string,
): unknown {
  const ctx = fakeSerializeAst(ast);
  const raw = parseSelector(selector, toElem, toAttr)[0];
  const sel = compileSelector(raw);

  return visit(ctx, sel, 0);
}

Deno.test("select descendant: A B", () => {
  const ast: TestNode = {
    type: "Foo",
    children: [{ type: "Bar" }, { type: "Baz" }],
  };

  expect(testSelector(ast, "Foo")).toEqual(ast);
  expect(testSelector(ast, "Foo Bar")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo Baz")).toEqual(ast.children![1]);

  // Not matching
  expect(testSelector(ast, "Foo Foo")).toEqual(undefined);
});

Deno.test("select child: A > B", () => {
  const ast: TestNode = {
    type: "Foo",
    foo: "fail",
    children: [{ type: "Bar", children: [{ type: "Foo" }] }, {
      type: "Foo",
    }],
  };
  expect(testSelector(ast, "Foo > Foo")).toEqual(ast.children![1]);
  expect(testSelector(ast, "Foo>Foo")).toEqual(ast.children![1]);
  expect(testSelector(ast, "* > Foo")).toEqual(ast.children![1]);
  expect(testSelector(ast, "*> Foo")).toEqual(ast.children![1]);
  expect(testSelector(ast, "* > *> Foo")).toEqual(ast.children![1]);
});

Deno.test("select child: A > B #2", () => {
  const ast: TestNode = {
    type: "Foo",
    foo: "fail",
    children: [{ type: "Bar", children: [{ type: "Foo" }] }],
  };
  expect(testSelector(ast, "Foo > Foo")).toEqual(undefined);
  expect(testSelector(ast, "Foo>Foo")).toEqual(undefined);
});

Deno.test("select child: A + B", () => {
  const ast: TestNode = {
    type: "Foo",
    children: [
      { type: "Bar", msg: "FAIL" },
      { type: "Bar", msg: "FAIL" },
      { type: "Baz" },
      { type: "Baz", msg: "FAIL" },
      { type: "Foo", msg: "FAIL" },
      { type: "Baz", msg: "FAIL" },
    ],
  };
  expect(testSelector(ast, "Bar + Baz")).toEqual(ast.children![2]);
  expect(testSelector(ast, "Bar+Baz")).toEqual(ast.children![2]);
});

Deno.test("select child: A ~ B", () => {
  const ast: TestNode = {
    type: "Foo",
    children: [
      { type: "Bar", msg: "FAIL" },
      { type: "Bar", msg: "FAIL" },
      { type: "Foo", msg: "FAIL" },
      { type: "Baz", msg: "ok #1" },
    ],
  };
  expect(testSelector(ast, "Bar ~ Baz")).toEqual(ast.children![3]);
  expect(testSelector(ast, "Bar~Baz")).toEqual(ast.children![3]);
});

Deno.test("select child: A[attr]", () => {
  const ast: TestNode = {
    type: "Foo",
    children: [
      { type: "Foo", msg: "a" },
      { type: "Bar", msg: "b" },
      { type: "Baz" },
    ],
  };
  expect(testSelector(ast, "[msg]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg=a]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg = a]")).toEqual(ast.children![0]);

  expect(testSelector(ast, "Foo[msg='a']")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg = 'a']")).toEqual(ast.children![0]);
  expect(testSelector(ast, 'Foo[msg="a"]')).toEqual(ast.children![0]);
  expect(testSelector(ast, 'Foo[msg = "a"]')).toEqual(ast.children![0]);
});

Deno.test("select child: A[attr <op> value]", () => {
  const ast: TestNode = {
    type: "Foo",
    children: [
      { type: "Foo", msg: false },
      { type: "Foo", msg: true },
      { type: "Foo", msg: 1 },
      { type: "Foo", msg: 2 },
    ],
  };
  expect(testSelector(ast, "Foo[msg=true]")).toEqual(ast.children![1]);
  expect(testSelector(ast, "Foo[msg = true]")).toEqual(ast.children![1]);
  expect(testSelector(ast, "Foo[msg=false]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg = false]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg=1]")).toEqual(ast.children![2]);
  expect(testSelector(ast, "Foo[msg = 1]")).toEqual(ast.children![2]);

  expect(testSelector(ast, "Foo[msg!=true]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg != true]")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo[msg!=false]")).toEqual(ast.children![1]);
  expect(testSelector(ast, "Foo[msg != false]")).toEqual(ast.children![1]);

  expect(testSelector(ast, "Foo[msg<2]")).toEqual(ast.children![2]);
  expect(testSelector(ast, "Foo[msg < 2]")).toEqual(ast.children![2]);

  expect(testSelector(ast, "Foo[msg<=1]")).toEqual(ast.children![2]);
  expect(testSelector(ast, "Foo[msg <= 1]")).toEqual(ast.children![2]);

  expect(testSelector(ast, "Foo[msg>1]")).toEqual(ast.children![3]);
  expect(testSelector(ast, "Foo[msg > 1]")).toEqual(ast.children![3]);
  expect(testSelector(ast, "Foo[msg>=1]")).toEqual(ast.children![2]);
  expect(testSelector(ast, "Foo[msg >= 1]")).toEqual(ast.children![2]);
});

Deno.test("select child: A:first-child", () => {
  const ast: TestNode = {
    type: "Foo",
    children: [
      { type: "Foo", msg: "a" },
      { type: "Foo", msg: "b" },
      { type: "Foo", msg: "c" },
      { type: "Foo", msg: "d" },
    ],
  };
  expect(testSelector(ast, "Foo:first-child")).toEqual(ast.children![0]);
  expect(testSelector(ast, "Foo:last-child")).toEqual(ast.children!.at(-1));
});

Deno.test("select child: A:nth-child", () => {
  const ast: TestNode = {
    type: "Bar",
    children: [
      { type: "Foo", msg: "a" },
      { type: "Foo", msg: "b" },
      { type: "Foo", msg: "c" },
      { type: "Foo", msg: "d" },
    ],
  };
  expect(testSelector(ast, "Foo:nth-child(2)")).toEqual(ast.children![1]);
  expect(testSelector(ast, "Foo:nth-child(2n)")).toEqual(ast.children![1]);
  expect(testSelector(ast, "Foo:nth-child(2n + 3)")).toEqual(ast.children![2]);
  expect(testSelector(ast, "Foo:nth-child(2n - 1)")).toEqual(ast.children![0]);
  expect(testSelector(ast, ":nth-child(2n + 2 of Foo)")).toEqual(
    ast.children![1],
  );
});

Deno.test("splitSelectors", () => {
  expect(splitSelectors("foo")).toEqual(["foo"]);
  expect(splitSelectors("foo, bar")).toEqual(["foo", "bar"]);
  expect(splitSelectors("foo:f(bar, baz)")).toEqual(["foo:f(bar, baz)"]);
  expect(splitSelectors("foo:f(bar, baz), foobar")).toEqual([
    "foo:f(bar, baz)",
    "foobar",
  ]);
});