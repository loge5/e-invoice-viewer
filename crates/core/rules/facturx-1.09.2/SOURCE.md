# Factur-X 1.09.2 / ZUGFeRD 2.5.2 profile schemas and Schematron

- Source: FeRD (Forum elektronische Rechnung Deutschland, AWV e.V.), package
  `ZUGFeRD_2.5.2_EN` from
  https://www.ferd-net.de/en/downloads/publications/details/zugferd-252-english
  (download needs contact details); Factur-X 1.09.2, released 2026-08-04,
  effective 2026-09-01. Provided by the author on 2026-11-20.
- Files, copied unchanged from the package's `Schema/<n>_Factur-X_1.09.2_<PROFILE>/`
  directories into one directory per profile (`minimum`, `basic-wl`,
  `basic`, `en16931`, `extended`): the XSD set, the Schematron and its code
  list file (`FACTUR-X_<PROFILE>_codedb.xml`, read by the Schematron with
  `document()`). Not bundled: the generated XSLT, the Excel files, the
  full CII D22B schema, the documentation and the sample invoices.
- Licence: FeRD's page *Disclaimer and rights of use ZUGFeRD* puts the
  technical artefacts (schemas and Schematron) under the Apache License 2.0,
  see `LICENSE`. The code list files belong to the Schematron (spec 0027).
  ZUGFeRD and FeRD are trademarks of AWV e.V.

SHA-256 of the bundled files (checked by a test):

```
5d72dd3f0df511b2994ea41b199685f9cc538e9ce57d50c84d21988fe1455c6e  basic/Factur-X_1.09.2_BASIC_urn_un_unece_uncefact_data_standard_QualifiedDataType_100.xsd
2ca9ab167afd99ae0455db1d7119007ab1de27941752dcc55e9e24bec0511c61  basic/Factur-X_1.09.2_BASIC_urn_un_unece_uncefact_data_standard_ReusableAggregateBusinessInformationEntity_100.xsd
ad32b1828200bdb0afd24fbe92fe90880bf357c6a51a9da95cddbdf593e3e35b  basic/Factur-X_1.09.2_BASIC_urn_un_unece_uncefact_data_standard_UnqualifiedDataType_100.xsd
808f07370fbbeaaceabbd674ee4d7d2cb358894112366989ff7d600d0432d444  basic/Factur-X_1.09.2_BASIC.xsd
a150b48359a72e1c1629edddb147d5133aabd03adb6ce8b3d287d0c2a6936d21  basic/FACTUR-X_BASIC_codedb.xml
b2a84cee0e497733bf595c435db18ba720a9f2e16f6741a972667a2f277f6bed  basic/FACTUR-X_BASIC.sch
5d72dd3f0df511b2994ea41b199685f9cc538e9ce57d50c84d21988fe1455c6e  basic-wl/Factur-X_1.09.2_BASICWL_urn_un_unece_uncefact_data_standard_QualifiedDataType_100.xsd
17d5300dcb6d7c1d7ac70c4d28c89fd3b778abb820129ac320b9010d4dc6bed3  basic-wl/Factur-X_1.09.2_BASICWL_urn_un_unece_uncefact_data_standard_ReusableAggregateBusinessInformationEntity_100.xsd
2f4686c41172611f7fc2347a7103a12c01746da17a5f7eded53254ceeff2469f  basic-wl/Factur-X_1.09.2_BASICWL_urn_un_unece_uncefact_data_standard_UnqualifiedDataType_100.xsd
6cd8f22081b659ebeb0a86a741ed454d1771460b822579a55b43f5f0cdd81356  basic-wl/Factur-X_1.09.2_BASICWL.xsd
a34cd66dda00a9260368fa02dad97e966ded1c1ad100787a5318074328d20abb  basic-wl/FACTUR-X_BASIC-WL_codedb.xml
c32dd6eb4ef9c3e88772726ff86aca796476b719152edd6d95aa96ddb17a63f0  basic-wl/FACTUR-X_BASIC-WL.sch
4bd1753b1f078ecc50468ec90e505a63a72c6c1f2449a893f37c72a2e933fe04  en16931/Factur-X_1.09.2_EN16931_urn_un_unece_uncefact_data_standard_QualifiedDataType_100.xsd
d6bf8e38c3d5595c670655ccb7ee89655ae244943aaf09a31758160caabc7894  en16931/Factur-X_1.09.2_EN16931_urn_un_unece_uncefact_data_standard_ReusableAggregateBusinessInformationEntity_100.xsd
60ee4d11da40c7ec42a4da1fb77b5d7bbffcdce42afd863fcd9da9334765315c  en16931/Factur-X_1.09.2_EN16931_urn_un_unece_uncefact_data_standard_UnqualifiedDataType_100.xsd
cb98b34e59398fcd8d48f29963a3fd6f4c49af9a8307569f4a1ced634f1aa885  en16931/Factur-X_1.09.2_EN16931.xsd
70339a41d71b02bb97683f8eea04acbd764e077a88a446b32860d89c3351b62f  en16931/FACTUR-X_EN16931_codedb.xml
ef8c8380b499d1267b2dcf1fa709b45631a77d6d5f220c6cc2ade2fdade59473  en16931/FACTUR-X_EN16931.sch
ece13000d4f99c97ef09b909e9f8646d45415c46fd24d19d3c9b771c72ae2b34  extended/Factur-X_1.09.2_EXTENDED_urn_un_unece_uncefact_data_standard_QualifiedDataType_100.xsd
83cef3c48a6c83149733ffdced362e230e9ae482f62acdfb8c0fb3a919452fdb  extended/Factur-X_1.09.2_EXTENDED_urn_un_unece_uncefact_data_standard_ReusableAggregateBusinessInformationEntity_100.xsd
c6c9f7b3f36c821999f20d7b7fff30db64112bc7b2a8f132e3d048d859855d00  extended/Factur-X_1.09.2_EXTENDED_urn_un_unece_uncefact_data_standard_UnqualifiedDataType_100.xsd
2accff44dbdc405f38fd35d789ff5e6262396940356ed362009509cd024556c6  extended/Factur-X_1.09.2_EXTENDED.xsd
6e23aedbba0e3b32b3b1211d145186193215f56f755201555306c42327e75fbc  extended/FACTUR-X_EXTENDED_codedb.xml
5b9039e31f86e90f1a34925e84d3fad206efa3ca5639bb98a0cefd3661d3e36a  extended/FACTUR-X_EXTENDED.sch
c65c4929cb6f51d59aec498851a9026f0a915555555b1c3e5eafd99f191be5f5  minimum/Factur-X_1.09.2_MINIMUM_urn_un_unece_uncefact_data_standard_QualifiedDataType_100.xsd
8a9c1826ac156d8d2faf47de8dcd8a41fff54e6cf3ba1d44503876c086c7d9db  minimum/Factur-X_1.09.2_MINIMUM_urn_un_unece_uncefact_data_standard_ReusableAggregateBusinessInformationEntity_100.xsd
7110f93a32e20af6fbc3b8c308ec77f75be18bafca9a933584eb457a187bd539  minimum/Factur-X_1.09.2_MINIMUM_urn_un_unece_uncefact_data_standard_UnqualifiedDataType_100.xsd
313e86212eaf9921f8cfe040d2acb72f718a580db3c19b8c8b3ca0a6a02b4216  minimum/Factur-X_1.09.2_MINIMUM.xsd
5c575cf37533a12a758f08b6b9691841c1691f2af214f21e040a07ec892e1f20  minimum/FACTUR-X_MINIMUM_codedb.xml
c570a628498e0b633a3dd66dd745653a8a2bc66fc6ee346b1971b9dc2d5f6778  minimum/FACTUR-X_MINIMUM.sch
```

To update: replace the files and this note with a newer package, change the
version strings in `crates/core/src/validate/profile.rs` and run
`cargo test` (with `FERD_PACKAGE` set to the unpacked package for the
sample test).
