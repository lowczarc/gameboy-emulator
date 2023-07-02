import argparse

instructions = [
    { "opcode": "LD", "params": [
        { "type": ["r", "r"], "format": lambda args: [0b01000000 | (args[0] << 3) | args[1]] },
        { "type": ["r", "8b"], "format": lambda args: [0b00000110 | (args[0] << 3), args[1]] },
        { "type": ["r", "(HL)"], "format": lambda args: [0b01000110 | (args[0] << 3)] },
        { "type": ["(HL)", "r"], "format": lambda args: [0b01110000 | args[1]] },
        { "type": ["(HL)", "8b"], "format": lambda args: [0b00110110, args[1]] },
        { "type": ["A", "(BC)"], "format": lambda args: [0b00001010] },
        { "type": ["A", "(DE)"], "format": lambda args: [0b00011010] },
        { "type": ["(BC)", "A"], "format": lambda args: [0b00000010] },
        { "type": ["(DE)", "A"], "format": lambda args: [0b00010010] },
        { "type": ["A", "(nn)"], "format": lambda args: [0b11111010, args[1] & 0xff, args[1] >> 8] },
        { "type": ["(nn)", "A"], "format": lambda args: [0b11101010, args[0] & 0xff, args[0] >> 8] },
        { "type": ["A", "(C)"], "format": lambda args: [0b11110010] },
        { "type": ["(C)", "A"], "format": lambda args: [0b11100010] },
        { "type": ["A", "(n)"], "format": lambda args: [0b11110000, args[1]] },
        { "type": ["(n)", "A"], "format": lambda args: [0b11100000, args[0]] },
        { "type": ["A", "(HL-)"], "format": lambda args: [0b00111010] },
        { "type": ["(HL-)", "A"], "format": lambda args: [0b00110010] },
        { "type": ["A", "(HL+)"], "format": lambda args: [0b00101010] },
        { "type": ["(HL+)", "A"], "format": lambda args: [0b00100010] },
        { "type": ["rr", "16b"], "format": lambda args: [0b00000001 | (args[0] << 4), args[1] & 0xff, args[1] >> 8] },
        { "type": ["(nn)", "SP"], "format": lambda args: [0b00001000, args[0] & 0xff, args[0] >> 8] },
        { "type": ["SP", "HL"], "format": lambda args: [0b11111001] },
        { "type": ["HL", "8b"], "format": lambda args: [0b11111000, args[1]] },
    ]},
    { "opcode": "PUSH", "params": [
        { "type": ["rr"], "format": lambda args: [0b11000101 | (args[0] << 4)] },
    ]},
    { "opcode": "POP", "params": [
        { "type": ["rr"], "format": lambda args: [0b11000001 | (args[0] << 4)] },
    ]},
    { "opcode": "ADD", "params": [
        { "type": ["r"], "format": lambda args: [0b10000000 | (args[0])] },
        { "type": ["(HL)"], "format": lambda args: [0b10000110] },
        { "type": ["8b"], "format": lambda args: [0b11000110, args[0]] },
        { "type": ["SP", "8b"], "format": lambda args: [0b11101000, args[1]] },

        # TODO: HL,rr (x9 ?)
    ]},
    { "opcode": "ADC", "params": [
        { "type": ["r"], "format": lambda args: [0b10001000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b10001110] },
        { "type": ["8b"], "format": lambda args: [0b11001110, args[0]] },
    ]},
    { "opcode": "SUB", "params": [
        { "type": ["r"], "format": lambda args: [0b10010000 | (args[0])] },
        { "type": ["(HL)"], "format": lambda args: [0b10010110] },
        { "type": ["8b"], "format": lambda args: [0b11010110, args[0]] },
    ]},
    { "opcode": "SBC", "params": [
        { "type": ["r"], "format": lambda args: [0b10011000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b10011110] },
        { "type": ["8b"], "format": lambda args: [0b11011110, args[0]] },
    ]},
    { "opcode": "CP", "params": [
        { "type": ["r"], "format": lambda args: [0b10111000 | (args[0])] },
        { "type": ["(HL)"], "format": lambda args: [0b10111110] },
        { "type": ["8b"], "format": lambda args: [0b11111110, args[0]] },
    ]},
    { "opcode": "INC", "params": [
        { "type": ["r"], "format": lambda args: [0b00000100 | (args[0] << 3)] },
        { "type": ["(HL)"], "format": lambda args: [0b00110100] },

        # THIS ONE IS SPECULATIVE
        { "type": ["rr"], "format": lambda args: [0b00000011 | (args[0] << 4)] },
    ]},
    { "opcode": "DEC", "params": [
        { "type": ["r"], "format": lambda args: [0b00000101 | (args[0] << 3)] },
        { "type": ["(HL)"], "format": lambda args: [0b00110101] },

        # TODO: rr
    ]},
    { "opcode": "AND", "params": [
        { "type": ["r"], "format": lambda args: [0b10100000 | (args[0])] },
        { "type": ["(HL)"], "format": lambda args: [0b10100110] },
        { "type": ["8b"], "format": lambda args: [0b11100110, args[0]] },
    ]},
    { "opcode": "OR", "params": [
        { "type": ["r"], "format": lambda args: [0b10110000 | (args[0])] },
        { "type": ["(HL)"], "format": lambda args: [0b10110110] },
        { "type": ["8b"], "format": lambda args: [0b11110110, args[0]] },
    ]},
    { "opcode": "XOR", "params": [
        { "type": ["r"], "format": lambda args: [0b10101000 | (args[0])] },
        { "type": ["(HL)"], "format": lambda args: [0b10101110] },
        { "type": ["8b"], "format": lambda args: [0b11101110, args[0]] },
    ]},
    { "opcode": "CCF", "params": [
        { "type": [], "format": lambda args: [0b00111111] },
    ]},
    { "opcode": "SCF", "params": [
        { "type": [], "format": lambda args: [0b00110111] },
    ]},
    { "opcode": "DAA", "params": [
        { "type": [], "format": lambda args: [0b00100111] },
    ]},
    { "opcode": "CPL", "params": [
        { "type": [], "format": lambda args: [0b00101111] },
    ]},
    { "opcode": "JP", "params": [
        { "type": ["16b"], "format": lambda args: [0b11000011, args[0] & 0xff, args[0] >> 8] },
        { "type": ["HL"], "format": lambda args: [0b11101001] },
        { "type": ["cc", "16b"], "format": lambda args: [0b11000010 | (args[0] << 3), args[1] & 0xff, args[1] >> 8] }
    ]},
    { "opcode": "JR", "params": [
        { "type": ["8b"], "format": lambda args: [0b00011000, args[0]] },
        { "type": ["cc", "8b"], "format": lambda args: [0b00100000 | (args[0] << 3), args[1]] }
    ]},
    { "opcode": "CALL", "params": [
        { "type": ["16b"], "format": lambda args: [0b11001101, args[0] & 0xff, args[0] >> 8] },
        { "type": ["cc", "16b"], "format": lambda args: [0b11000100 | (args[0] << 3), args[1] & 0xff, args[1] >> 8] }
    ]},
    { "opcode": "RET", "params": [
        { "type": [], "format": lambda args: [0b11001001] },
        { "type": ["cc"], "format": lambda args: [0b11000000 | (args[0] << 3)] },
    ]},
    { "opcode": "RETI", "params": [
        { "type": [], "format": lambda args: [0b11011001] },
    ]},
    { "opcode": "RST", "params": [
        { "type": ["n"], "format": lambda args: [0b11000111 | (args[0] << 3)] },
    ]},
    { "opcode": "DI", "params": [
        { "type": [], "format": lambda args: [0b11110011] },
    ]},
    { "opcode": "EI", "params": [
        { "type": [], "format": lambda args: [0b11111011] },
    ]},
    { "opcode": "NOP", "params": [
        { "type": [], "format": lambda args: [0b00000000] },
    ]},
    { "opcode": "HALT", "params": [
        { "type": [], "format": lambda args: [0b01110110] },
    ]},
    { "opcode": "STOP", "params": [
        { "type": [], "format": lambda args: [0b00010000, 0b00000000] },
    ]},
    { "opcode": "RLCA", "params": [
        { "type": [], "format": lambda args: [0b00000111] },
    ]},
    { "opcode": "RLA", "params": [
        { "type": [], "format": lambda args: [0b00010111] },
    ]},
    { "opcode": "RRCA", "params": [
        { "type": [], "format": lambda args: [0b00001111] },
    ]},
    { "opcode": "RRA", "params": [
        { "type": [], "format": lambda args: [0b00011111] },
    ]},
    # THE NEXT ONES ARE SPECULATIVE BASED ON ASSEMBLED SOURCE CODE
    { "opcode": "BIT", "params": [
        { "type": ["n", "r"], "format": lambda args: [0b11001011, 0b00001000 | (args[0] << 4) | args[1]] },
        { "type": ["n", "(HL)"], "format": lambda args: [0b11001011, 0b00001110 | (args[0] << 4)] },
    ]},
    { "opcode": "RLC", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00000000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00000110] },
    ]},
    { "opcode": "RL", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00010000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00010110] },
    ]},
    { "opcode": "RRC", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00001000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00001110] },
    ]},
    { "opcode": "RR", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00011000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00011110] },
    ]},
    { "opcode": "SLA", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00100000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00100110] },
    ]},
    { "opcode": "SWAP", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00110000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00110110] },
    ]},
    { "opcode": "SRA", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00101000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00101110] },
    ]},
    { "opcode": "SRL", "params": [
        { "type": ["r"], "format": lambda args: [0b11001011, 0b00111000 | args[0]] },
        { "type": ["(HL)"], "format": lambda args: [0b11001011, 0b00111110] },
    ]},
    # TODO:
    #   set :(0xcb ?)
    #   res :(0xcb ?)

    { "opcode": ".DB", "params": [
     { "type": ["*"], "format": lambda args: args },
    ]},
]

registers = {
    "A": 7,
    "B": 0,
    "C": 1,
    "D": 2, # Maybe ?
    "E": 3,
    "H": 4, # Maybe ?
    "L": 5, # Maybe ?
    # (HL) is not a register but is associated with 6

    "BC": 0,
    "DE": 1,
    "HL": 2, # Confirmed
    "SP": 3, # Confirmed
}

conditions = {
    # Not checked:
    "NZ": 0,
    "Z": 1,
    "NC": 2,
    "C": 3,
}


class Param:
    def __init__(self, value, labels):
        self.type, self.value = self.get_type_value(value.upper().strip(), labels)

    def get_type_value(self, input, labels):
        if input in ['A', 'B', 'C', 'D', 'E', 'H', 'L']:
            return ['r', input], registers[input]
        elif input in ['BC', 'DE', 'HL', 'SP']:
            return ['rr', input], registers[input]
        elif len(input) == 4 and input[:2] == '0X':
            return ['8b'], int(input[2:], 16)
        elif len(input) == 6 and input[:2] == '0X':
            return ['16b'], int(input[2:], 16)
        elif len(input) == 8 and input[:3] == '(0X' and input[-1] == ')':
            return ['(nn)'], int(input[3:-1], 16)
        elif len(input) == 6 and input[:3] == '(0X' and input[-1] == ')':
            return ['(n)'], int(input[3:-1], 16)
        elif input in ['NZ', 'Z', 'NC', 'C']:
            return ['cc', input], conditions[input]
        elif input in ['0','1','2','3','4','5','6','7']:
            return ['n'], int(input)
        elif input in ["(HL)", "(BC)", "(DE)", "(C)", "(HL-)", "(HL+)"]:
            return [input], 0
        else:
            raise ValueError("Invalid parameter ({})".format(input))


class Instruction:
    def __init__(self, value, labels):
        splitted = value.split(' ')
        self.opcode = splitted[0].upper().strip()
        self.params = [Param(param, labels) for param in splitted[1:] if param.strip()]

    def get_instruction_format(self):
        for instruction in instructions:
            if self.opcode == instruction['opcode']:
                for params in instruction['params']:
                    if len(params["type"]) == 1 and params["type"][0] == "*":
                        return params
                    if len(params["type"]) == len(self.params):
                        for i in range(len(params["type"])):
                            if params["type"][i] not in self.params[i].type:
                                break
                        else:
                            return params
                return None

    def __str__(self):
        return "".join(["{:02x}".format(b) for b in self.to_bytes()])

    def to_bytes(self):
        instruction_format = self.get_instruction_format()['format']
        return instruction_format([param.value for param in self.params])

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input_file", help="The input file in gbasm")
    parser.add_argument("output_file", help="The output gb rom filename (e.g. basic.rom)")
    args = parser.parse_args()

    f = open(args.input_file, "r")

    starting_address = 0x0;
    lines = []
    labels = {}
    # Preprocess
    for line in f:
        line_without_comment = line.split(';')[0].strip()

        if ':' in line_without_comment:
            splitted = line_without_comment.split(':')
            labels[splitted[0].strip()] = starting_address
            line_without_comment = splitted[1].strip()

        if line_without_comment != '':
            lines.append(line_without_comment.replace("$", "0x").replace(",", " ").replace("0xFF00+", ""))
            starting_address += 2

    print(labels)
    program = []
    # Compile
    for line in lines:
        instruction = Instruction(line, labels)
        print("Instruction: " + instruction.opcode + " " + str(instruction.params))
        print("Valid: " + str(instruction.get_instruction_format()))
        print("Format: " + str(instruction))

        program += instruction.to_bytes()

    output = open(args.output_file, "wb")
    output.write(bytearray(program))
    output.close()

if __name__ == "__main__":
    main()
