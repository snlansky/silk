use rcgen::{BasicConstraints, Certificate, CertificateParams, DnType, IsCa, PKCS_ED25519};
use webpki::{EndEntityCert, TLSServerTrustAnchors};
use webpki::trust_anchor_util::cert_der_as_trust_anchor;
use webpki::SignatureAlgorithm;
use webpki::{Time, DNSNameRef};





const SILK_ORG: &str = "www.silk.org";
const SILK_COMMON: &str = "www.silk.com";


pub fn ed25519_cert_params(domain: String, org_name:String, common_name: String) -> CertificateParams {
    let mut params = CertificateParams::new(vec![domain]);

    params.distinguished_name.push(DnType::OrganizationName, org_name);
    params.distinguished_name.push(DnType::CommonName, common_name);
    params.alg = &PKCS_ED25519;
    params
}

fn check_cert<'a, 'b>(cert_der :&[u8], cert :&'a Certificate, alg :&SignatureAlgorithm,
                      sign_fn :impl FnOnce(&'a Certificate, &'b [u8]) -> Vec<u8>) {
    println!("{}", cert.serialize_pem().unwrap());
    let trust_anchor = cert_der_as_trust_anchor(&cert_der).unwrap();
    let trust_anchor_list = &[trust_anchor];
    let trust_anchors = TLSServerTrustAnchors(trust_anchor_list);
    let end_entity_cert = EndEntityCert::from(&cert_der).unwrap();

    // Set time to Jan 10, 2004
    let time = Time::from_seconds_since_unix_epoch(0x40_00_00_00);

    // (1/3) Check whether the cert is valid
    end_entity_cert.verify_is_valid_tls_server_cert(
        &[&alg],
        &trust_anchors,
        &[],
        time,
    ).expect("valid TLS server cert");

    // (2/3) Check that the cert is valid for the given DNS name
    let dns_name = DNSNameRef::try_from_ascii_str("crabs.crabs").unwrap();
    end_entity_cert.verify_is_valid_for_dns_name(
        dns_name,
    ).expect("valid for DNS name");

    // (3/3) Check that a message signed by the cert is valid.
    let msg = b"Hello, World! This message is signed.";
    let signature = sign_fn(&cert, msg);
    end_entity_cert.verify_signature(
        &alg,
        msg,
        &signature,
    ).expect("signature is valid");
}

pub fn default_params() -> CertificateParams {
    let mut params = CertificateParams::new(vec![
        "crabs.crabs".to_string(), "localhost".to_string(),
    ]);

    params.distinguished_name.push(DnType::OrganizationName, "Crab widgits SE");
    params.distinguished_name.push(DnType::CommonName, "Master CA");
    // params.alg = &PKCS_ED25519;
    params
}



#[cfg(test)]
mod tests {
    use crate::cert::default_params;
    use rcgen::{Certificate, IsCa, BasicConstraints, CertificateParams, DnType, PKCS_ED25519};
    
    use ring::signature::{EcdsaKeyPair, EcdsaSigningAlgorithm,
                          Ed25519KeyPair, RSA_PKCS1_SHA256, RsaKeyPair};
    use webpki::{EndEntityCert, ED25519, TLSServerTrustAnchors, Time};
    use webpki::trust_anchor_util::cert_der_as_trust_anchor;

    #[test]
    fn test_cert() {
        let mut params = default_params();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let ca_cert = Certificate::from_params(params).unwrap();

        let ca_der = ca_cert.serialize_der().unwrap();
        let trust_anchor_list = &[cert_der_as_trust_anchor(&ca_der).unwrap()];
        let trust_anchors = TLSServerTrustAnchors(trust_anchor_list);

        let mut params = CertificateParams::new(vec!["crabs.dev".to_string()]);
        params.distinguished_name.push(DnType::OrganizationName, "Crab widgits SE");
        params.distinguished_name.push(DnType::CommonName, "Dev domain");
        let cert = Certificate::from_params(params).unwrap()
            .serialize_der_with_signer(&ca_cert).unwrap();
        let cert_new = Certificate::from_params(default_params()).unwrap()
            .serialize_pem_with_signer(&ca_cert).unwrap();
        println!("new cert-> \n{}", cert_new);
        let end_entity_cert = EndEntityCert::from(&cert).unwrap();

        // Set time to Jan 10, 2004
        let time = Time::from_seconds_since_unix_epoch(0x40_00_00_00);

        end_entity_cert.verify_is_valid_tls_server_cert(&[&webpki::ECDSA_P256_SHA256], &trust_anchors ,&[&ca_der], time).unwrap();
    }

    #[test]
    fn test_openssl() {
        let mut params = default_params();
        params.alg = &PKCS_ED25519;
        let cert = Certificate::from_params(params).unwrap();


        let pk_der = cert.serialize_private_key_der();

        let key_pair = Ed25519KeyPair::from_pkcs8_maybe_unchecked(&pk_der).unwrap();
        let signature = key_pair.sign("hello world".as_bytes());
        let v = signature.as_ref().to_vec();
        // println!("-- {:?}", v);
        let der = cert.serialize_der().unwrap();
        let end_entity_cert = EndEntityCert::from(&der).unwrap();
        end_entity_cert.verify_signature(&ED25519, "hello world".as_bytes(), &v).unwrap();

        // ED25519.verify(key_pair.public_key().into(), "hello world".as_bytes().into(), v.as_slice().into());

// Now verify the certificate.
        let pem = cert.serialize_pem().unwrap();
        let pair = cert.get_key_pair();


        println!("{}", pem);
        println!("{}", pair.serialize_pem());
        println!("{}", pair.public_key_pem());
    }

    #[test]
    fn test_1() {
        // let key = crate::ed25519::Keypair::generate();
        // let buf = key.encode();
        // let kp = rcgen::KeyPair::try_from(&buf[..]).unwrap();
        let _sk = "MFMCAQEwBQYDK2VwBCIEIB/THOx4R6EF3m5GPzWw9y2ojw6czaurz1IiaZJR3Aq9oSMDIQA8URGFcDwhxtCb+rvt7LriOtqr1mZf+lTsDG+haxgXLw==";

    }
}
