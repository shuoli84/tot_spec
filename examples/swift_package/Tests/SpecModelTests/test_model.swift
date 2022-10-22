import Foundation
import XCTest
import SpecModel

class TestModels: XCTestCase {
    func testModels() {
        let encoder = JSONEncoder()
        encoder.outputFormatting = .prettyPrinted
        
        let decoder = JSONDecoder()
        
        do {
            let data = SimpleStruct(bool_value: true, i8_value: 123, i64_value: 3333)
            let json_data = try! encoder.encode(data)
            print(String(data: json_data, encoding: .utf8)!)
        }
        
        do {
            let json_data = try! encoder.encode(Number.I64(23))
            print(String(data: json_data, encoding: .utf8)!)
            
            let data_back = try! decoder.decode(Number.self, from: json_data)
            print("data back \(data_back)")
        }
        
        do {
            let json_data = try! encoder.encode(Number.F64(23.123))
            print(String(data: json_data, encoding: .utf8)!)
            
            let data_back = try! decoder.decode(Number.self, from: json_data)
            print("data back \(data_back)")
        }
        
        do {
            let json_data = try! encoder.encode(ResetRequest(request_id: "hello"))
            print(String(data: json_data, encoding: .utf8)!)
            
            let data_back = try! decoder.decode(ResetRequest.self, from: json_data)
            print("data back \(data_back)")
        }

        print("\(SpecModel.ConstInteger.Value1) = \(SpecModel.ConstInteger.Value1.rawValue)")
    }
}
