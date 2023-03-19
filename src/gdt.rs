use log::error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::result::Result;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct GdtFile {
    pub record_type: u32,
    pub record_length: u32,
    pub object_request: GdtRequestObject, // Obj_Anforderung
    pub object_annex: GdtAnnexObject,     // Obj_Anhang
    pub object_physician_identification: GdtPhysicianIdentificationObject, // Obj_Arztidentifikation
    pub object_basic_diagnostics: GdtBasicDiagnosticsObject, // Obj_Basisdiagnostikdia
    pub object_permanent_diagnosis: GdtPermanentDiagnosisObject, // Obj_Dauerdiagnosis
    pub object_permanent_medication: GdtPermanentMedicationObject, // Obj_Dauermedikament
    pub object_diagnosis: GdtDiagnosisObject, // Obj_Diagnosis
    pub object_admission: GdtAdmissionObject, // Obj_Einweisung
    pub object_header_data: GdtHeaderDataObject, // Obj_Kopfdaten
    pub object_patient: GdtPatientObject, // Obj_Patient
    pub object_invoice_recipient: GdtInvoiceRecipientObject, // Obj_RgEmpfänger
    pub object_end_of_record: GdtEndOfRecordObject, // Obj_Satzende
    pub object_certificate: GdtCertificateObject, // Obj_Schein
    pub object_appointment_request: GdtAppointmentRequestObject, // Obj_Terminanfrage
    pub object_referral: GdtReferralObject, // Obj_Ueberweisung
    pub object_health: GdtHealthObject,   // Obj_Versichertenkarte
}

#[derive(Debug, Default)]
pub struct GdtRequestObject {
    date_of_examination: String, // 6200, MMDDYYYY
    time_of_examination: String, // 6201, e.g. 110435
    request_identifier: String,  // 8310
    request_uid: String,         // 8314
}

#[derive(Debug, Default)]
pub struct GdtAnnexObject {}

#[derive(Debug, Default)]
pub struct GdtPhysicianIdentificationObject {}

#[derive(Debug, Default)]
pub struct GdtBasicDiagnosticsObject {
    pub patient_height: String, // 3622, cm
    pub patient_weight: String, // 3623 (but the example in PDF says 3632), kg
}

#[derive(Debug, Default)]
pub struct GdtPermanentDiagnosisObject {}

#[derive(Debug, Default)]
pub struct GdtPermanentMedicationObject {}

#[derive(Debug, Default)]
pub struct GdtDiagnosisObject {}

#[derive(Debug, Default)]
pub struct GdtAdmissionObject {}

#[derive(Debug, Default)]
pub struct GdtHeaderDataObject {
    pub gdt_id_receiver: String, // 8315, GDT-ID of the receiver
    pub gdt_id_sender: String,   // 8316, GDT-ID of the sender
    pub version_gdt: String,     // 9218, Version GDT
}

#[derive(Debug, Default)]
pub struct GdtPatientObject {
    pub patient_number: String,           // 3000
    pub patient_name: String,             // 3101
    pub patient_first_name: String,       // 3102
    pub patient_dob: String,              // 3103, MMDDYYYY
    pub patient_gender: GdtPatientGender, // 3110
}

#[derive(Debug, Default)]
pub struct GdtInvoiceRecipientObject {}

#[derive(Debug, Default)]
pub struct GdtEndOfRecordObject {}

#[derive(Debug, Default)]
pub struct GdtCertificateObject {}

#[derive(Debug, Default)]
pub struct GdtAppointmentRequestObject {}

#[derive(Debug, Default)]
pub struct GdtReferralObject {}

#[derive(Debug, Default)]
pub struct GdtHealthObject {}

#[derive(Debug)]
pub enum GdtPatientGender {
    Male,
    Female,
}

impl Default for GdtPatientGender {
    fn default() -> GdtPatientGender {
        GdtPatientGender::Male
    }
}

#[derive(Debug)]
pub struct RawGdtLine {
    field_identifier: u32,
    content: String,
}

#[derive(Debug)]
pub enum GdtError {
    IoError(std::io::Error),
    FieldIdentifierNotNumber(String, std::num::ParseIntError),
    LineTooShort(String),
    LineNotFound(String),
    NumberExpected(String, std::num::ParseIntError),
    InvalidValue(String, String),
}

pub fn parse_file_lines<P>(
    path: P,
) -> Result<impl std::iter::Iterator<Item = Result<RawGdtLine, GdtError>>, GdtError>
where
    P: AsRef<Path>,
{
    let file = File::open(path).map_err(GdtError::IoError)?;
    let reader = BufReader::new(file);

    return Ok(reader.lines().map(|r_str| {
        r_str
            .map_err(GdtError::IoError)
            .and_then(string_to_gdt_line)
    }));
}

fn string_to_gdt_line(str: String) -> Result<RawGdtLine, GdtError> {
    if str.len() < 7 {
        return Err(GdtError::LineTooShort(str));
    }
    let field_id = u32::from_str(&str[3..7])
        .map_err(|e| GdtError::FieldIdentifierNotNumber(String::from(&str), e))?;

    return Ok(RawGdtLine {
        field_identifier: field_id,
        content: String::from(&str[7..]),
    });
}

pub fn parse_file<P>(path: P) -> Result<GdtFile, GdtError>
where
    P: AsRef<Path>,
{
    let mut file = Default::default();
    let mut iter = parse_file_lines(path)?;
    read_record_header(&mut file, &mut iter)?;
    while let Some(r_next_line) = iter.next() {
        match r_next_line {
            Err(e) => error!("Error in line: {:?}", e),
            Ok(RawGdtLine {
                field_identifier: 8200,
                content,
            }) if content.as_str() == "Obj_Anforderung" => {
                file.object_request = read_request_object(&mut iter)?;
            }
            Ok(RawGdtLine {
                field_identifier: 8200,
                content,
            }) if content.as_str() == "Obj_Kopfdaten" => {
                file.object_header_data = read_header_data_object(&mut iter)?;
            }
            Ok(RawGdtLine {
                field_identifier: 8200,
                content,
            }) if content.as_str() == "Obj_Patient" => {
                file.object_patient = read_patient_object(&mut iter)?;
            }
            Ok(RawGdtLine {
                field_identifier: 8200,
                content,
            }) if content.as_str() == "Obj_Basisdiagnostik" => {
                file.object_basic_diagnostics = read_basic_diagnostics_object(&mut iter)?;
            }
            Ok(RawGdtLine {
                field_identifier: 8202,
                ..
            }) => {
                return Ok(file);
            }
            _ => {}
        }
    }
    return Ok(file);
}

type GdtLineIter<'a> = dyn 'a + std::iter::Iterator<Item = Result<RawGdtLine, GdtError>>;

fn read_record_header(file: &mut GdtFile, iter: &mut GdtLineIter) -> Result<(), GdtError> {
    let first_line = iter
        .next()
        .ok_or_else(|| GdtError::LineNotFound("0".to_string()))
        .and_then(|x| x)?;
    let first_line_content = u32::from_str(&first_line.content)
        .map_err(|e| GdtError::NumberExpected(first_line.content, e))?;

    let second_line = iter
        .next()
        .ok_or_else(|| GdtError::LineNotFound("1".to_string()))
        .and_then(|x| x)?;
    let second_line_content = u32::from_str(&second_line.content)
        .map_err(|e| GdtError::NumberExpected(second_line.content, e))?;

    file.record_type = first_line_content;
    file.record_length = second_line_content;
    return Ok(());
}

fn read_request_object(iter: &mut GdtLineIter) -> Result<GdtRequestObject, GdtError> {
    let mut obj: GdtRequestObject = Default::default();
    while let Some(r_next_line) = iter.next() {
        match r_next_line {
            Err(e) => error!("Error in object: {:?}", e),
            Ok(RawGdtLine {
                field_identifier: 6200,
                content,
            }) => {
                obj.date_of_examination = content;
            }
            Ok(RawGdtLine {
                field_identifier: 6201,
                content,
            }) => {
                obj.time_of_examination = content;
            }
            Ok(RawGdtLine {
                field_identifier: 8310,
                content,
            }) => {
                obj.request_identifier = content;
            }
            Ok(RawGdtLine {
                field_identifier: 8314,
                content,
            }) => {
                obj.request_uid = content;
            }
            Ok(RawGdtLine {
                field_identifier: 8201,
                ..
            }) => {
                return Ok(obj);
            }
            _ => {}
        }
    }
    return Ok(obj);
}

fn read_header_data_object(iter: &mut GdtLineIter) -> Result<GdtHeaderDataObject, GdtError> {
    let mut obj: GdtHeaderDataObject = Default::default();
    while let Some(r_next_line) = iter.next() {
        match r_next_line {
            Err(e) => error!("Error in object: {:?}", e),
            Ok(RawGdtLine {
                field_identifier: 8315,
                content,
            }) => {
                obj.gdt_id_receiver = content;
            }
            Ok(RawGdtLine {
                field_identifier: 8316,
                content,
            }) => {
                obj.gdt_id_sender = content;
            }
            Ok(RawGdtLine {
                field_identifier: 9218,
                content,
            }) => {
                obj.version_gdt = content;
            }
            Ok(RawGdtLine {
                field_identifier: 8201,
                ..
            }) => {
                return Ok(obj);
            }
            _ => {}
        }
    }
    return Ok(obj);
}

fn read_patient_object(iter: &mut GdtLineIter) -> Result<GdtPatientObject, GdtError> {
    let mut obj: GdtPatientObject = Default::default();
    while let Some(r_next_line) = iter.next() {
        match r_next_line {
            Err(e) => error!("Error in object: {:?}", e),
            Ok(RawGdtLine {
                field_identifier: 3000,
                content,
            }) => {
                obj.patient_number = content;
            }
            Ok(RawGdtLine {
                field_identifier: 3101,
                content,
            }) => {
                obj.patient_name = content;
            }
            Ok(RawGdtLine {
                field_identifier: 3102,
                content,
            }) => {
                obj.patient_first_name = content;
            }
            Ok(RawGdtLine {
                field_identifier: 3103,
                content,
            }) => {
                obj.patient_dob = content;
            }
            Ok(RawGdtLine {
                field_identifier: 3110,
                content,
            }) => {
                if content == "1" {
                    obj.patient_gender = GdtPatientGender::Male;
                } else if content == "2" {
                    obj.patient_gender = GdtPatientGender::Female;
                } else {
                    return Err(GdtError::InvalidValue("Gender".to_string(), content));
                }
            }
            Ok(RawGdtLine {
                field_identifier: 8201,
                ..
            }) => {
                return Ok(obj);
            }
            _ => {}
        }
    }
    return Ok(obj);
}

fn read_basic_diagnostics_object(
    iter: &mut GdtLineIter,
) -> Result<GdtBasicDiagnosticsObject, GdtError> {
    let mut obj: GdtBasicDiagnosticsObject = Default::default();
    while let Some(r_next_line) = iter.next() {
        match r_next_line {
            Err(e) => error!("Error in object: {:?}", e),
            Ok(RawGdtLine {
                field_identifier: 3622,
                content,
            }) => {
                obj.patient_height = content;
            }
            Ok(RawGdtLine {
                field_identifier: 3623,
                content,
            }) => {
                obj.patient_weight = content;
            }
            Ok(RawGdtLine {
                field_identifier: 3632,
                content,
            }) => {
                obj.patient_weight = content;
            }
            Ok(RawGdtLine {
                field_identifier: 8201,
                ..
            }) => {
                return Ok(obj);
            }
            _ => {}
        }
    }
    return Ok(obj);
}
