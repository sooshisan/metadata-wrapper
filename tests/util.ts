import { expect } from "chai";

export const expectThrowsAsync = async (
  method: () => Promise<any>,
  errorMessage = undefined
) => {
  let error: unknown = null;
  try {
    await method();
  } catch (err: unknown) {
    error = err;
  }
  expect(error).to.be.an("Error");
  if (errorMessage) {
    expect((error as any).message).to.equal(errorMessage);
  }
};
