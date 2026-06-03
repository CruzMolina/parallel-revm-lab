{
  accesses: [],
  faults: [],
  warnings: [
    "Geth JavaScript tracer records SLOAD/SSTORE storage observations only.",
    "Provider support for debug_traceTransaction and JavaScript tracers varies."
  ],

  byteHex: function (byte) {
    var value = byte.toString(16);
    return value.length === 1 ? "0" + value : value;
  },

  bytesHex: function (bytes) {
    var out = "";
    for (var i = 0; i < bytes.length; i++) {
      out += this.byteHex(bytes[i]);
    }
    return "0x" + out;
  },

  wordHex: function (word) {
    var out = word.toString(16);
    while (out.length < 64) {
      out = "0" + out;
    }
    return "0x" + out;
  },

  currentAddress: function (log) {
    return this.bytesHex(log.contract.getAddress());
  },

  pushStorage: function (log, kind, op, slot) {
    this.accesses.push({
      address: this.currentAddress(log),
      slot: slot,
      kind: kind,
      op: op,
      pc: log.getPC(),
      depth: log.getDepth(),
      gas_remaining: log.getGas()
    });
  },

  step: function (log, db) {
    var err = log.getError();
    if (err) {
      this.faults.push({ pc: log.getPC(), error: err.toString() });
      return;
    }

    var opcode = log.op.toNumber();
    if (opcode === 0x54) {
      this.pushStorage(log, "read", "SLOAD", this.wordHex(log.stack.peek(0)));
    } else if (opcode === 0x55) {
      this.pushStorage(log, "write", "SSTORE", this.wordHex(log.stack.peek(0)));
    }
  },

  fault: function (log, db) {
    this.faults.push({
      pc: log.getPC(),
      gas_remaining: log.getGas(),
      error: log.getError() ? log.getError().toString() : "fault"
    });
  },

  result: function (ctx, db) {
    return {
      accesses: this.accesses,
      faults: this.faults,
      warnings: this.warnings
    };
  }
}
