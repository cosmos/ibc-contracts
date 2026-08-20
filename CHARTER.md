# Technical Charter (the "Charter") for IBC a Series of Cosmos Projects, LLC

**Adopted \_\_\_\_\_\_\_\_\_\_\_\_, 2026**

This Charter sets forth the responsibilities and procedures for technical contribution to, and oversight of, the IBC open source project, which has been established as IBC a Series of Cosmos Projects, LLC (the "Project"). Cosmos Projects, LLC ("Cosmos Projects") is a Delaware series limited liability company. All contributors (including committers, maintainers, and other technical positions) and other participants in the Project (collectively, "Collaborators") must comply with the terms of this Charter.

1. **Mission and Scope of the Project**

      1. The mission of the Project is to develop and maintain IBC, an open standard and its reference implementations for trust-minimized interoperability between independent distributed ledgers.

      2. The scope of the Project includes collaborative development under the Project License (as defined herein) supporting the mission, including the protocol specifications, implementations, documentation, testing, integration and the creation of other artifacts that aid the development, deployment, operation or adoption of the open source project.

2. **Technical Steering Committee**

      1. The Technical Steering Committee (the "TSC") will be responsible for the technical oversight of the open source Project that is expressly reserved to it under Section 2.g. All matters not so reserved default to the Maintainers within their respective domains.

      2. The TSC will consist of five seats: three seats elected by the Maintainers from among the Maintainers, and two seats elected by the Committers from among the Committers. The TSC voting members at the inception of the Project, and going forward, will be as set forth within the "TSC_MEMBERS" file within the Project's code repository. Any meetings of the TSC are intended to be open to the public, and can be conducted electronically, via teleconference, or in person.

      3. No more than two of the five TSC seats may be held at any time by Related Parties, where "Related Parties" means any two or more persons employed by, or acting on behalf of, the same entity or its affiliates. This limitation does not apply to the initial TSC named under Section 2.d. and takes effect at the first election held after the initial term.

      4. The initial TSC will serve a term of two years from the adoption of this Charter, after which the first election will be held. Subsequent terms and election procedures will be established by the TSC and documented in the TSC_MEMBERS file.

      5. The TSC may elect a Chair, who will preside over meetings of the TSC and will serve until their resignation or replacement by the TSC. The Chair, or any other TSC member so designated by the TSC, will serve as the primary communication contact for the Project.

      6. The Project generally will involve Contributors, Committers, and Maintainers. The TSC may adopt or modify roles so long as the roles are documented in the MAINTAINERS file. Unless otherwise documented:

         1. Contributors include anyone in the technical community that contributes code, documentation, specifications, or other technical artifacts to the Project;

         2. Committers are Contributors who have earned the ability to modify ("commit") source code, documentation or other technical artifacts in a project's repository;

         3. Maintainers are Committers who additionally hold the governance franchise, and who vote on the promotion and removal of Committers and Maintainers and on TSC representation; and

         4. A Contributor may become a Committer, and a Committer may become a Maintainer, by following the previously agreed process outlined in [MAINTAINERS.md](MAINTAINERS.md).

      7. Responsibilities: the matters reserved to the TSC are limited to the following:

         1. coordinating the overall technical direction of the Project;

         2. approving the creation, scope change, and archiving of sub-projects;

         3. creating sub-committees or working groups to focus on cross-project technical issues and requirements;

         4. appointing representatives to work with other open source or open standards communities;

         5. establishing community norms, workflows, release policies, and security issue reporting and disclosure policies;

         6. approving and implementing policies and processes for contributing, to be published in the CONTRIBUTING file, and coordinating with the series manager of the Project (as provided for in the Series Agreement, the "Series Manager") to resolve matters or concerns that may arise as set forth in Section 7 of this Charter;

         7. resolving technical disputes that the Maintainers have been unable to resolve among themselves, and that are genuinely cross-cutting in nature; and

         8. coordinating communications regarding the Project.

      8. Participation in the Project through becoming a Contributor, Committer, or Maintainer is open to anyone so long as they abide by the terms of this Charter.

      9. Requirements and prioritized use cases originating from external working groups or other bodies enter the Project as ordinary public proposals and carry no special standing. Such bodies are outside the scope of the Project and are not governed by this Charter.

3. **Voting**

   1. While the Project aims to operate as a consensus-based community, decisions proceed by lazy consensus wherever practicable: a proposal proceeds unless a Collaborator with standing objects within the stated window. If any decision of the TSC requires a vote to move the Project forward, the voting members of the TSC will vote on a one vote per voting member basis. Unless otherwise stated, a 51% vote is required for such decisions that cannot be made by rough consensus.

   2. It is anticipated that most decisions will be made asynchronously, but meetings of the TSC may be held. Quorum for such meetings requires at least fifty percent of all voting members of the TSC to be present. Meetings of the TSC may continue if quorum is not met but will be prevented from making any decisions at the meeting.

   3. Except as provided in Section 7.c. and 8.a., decisions by vote at a meeting require a majority vote of those in attendance, provided quorum is met. Decisions made asynchronously by electronic vote without a meeting require a majority vote of the voting members of the TSC.

   4. In the event a vote cannot be resolved by the TSC, any voting member of the TSC may refer the matter to the Series Manager for assistance in reaching a resolution.

4. **Compliance with Policies**

   1. This Charter is subject to the Series Agreement for the Project and the Operating Agreement of Cosmos Projects. Contributors will comply with the policies of Cosmos Projects as currently adopted by Cosmos Projects, including the policies listed at [https://cosmoslabs.io/policies/](https://cosmoslabs.io/policies/).

   2. The TSC may adopt a code of conduct ("CoC") for the Project, which is subject to approval by the Series Manager. In the event that a Project-specific CoC has not been approved, the Cosmos Projects Code of Conduct listed at [https://cosmoslabs.io/policies](https://cosmoslabs.io/policies) will apply for all Collaborators in the Project.

   3. When amending or adopting any policy applicable to the Project, Cosmos Projects will publish such policy, as to be amended or adopted, on its web site at least 30 days prior to such policy taking effect; provided, however, that in the case of any amendment of the Trademark Policy or Terms of Use of Cosmos Projects, any such amendment is effective upon publication on Cosmos Project's web site.

   4. All Collaborators must allow open participation from any individual or organization meeting the requirements for contributing under this Charter and any policies adopted for all Collaborators by the TSC, regardless of competitive interests. Put another way, the Project community must not seek to exclude any participant based on any criteria, requirement, or reason other than those that are reasonable and applied on a non-discriminatory basis to all Collaborators in the Project community.

   5. The Project will operate in a transparent, open, collaborative, and ethical manner at all times. The output of all Project discussions, proposals, timelines, decisions, and status should be made open and easily visible to all. The single exception is the handling of security vulnerability reports, which follow the coordinated disclosure process published in the SECURITY file and may remain confidential within the TSC and its designees while a fix is prepared. Any potential violations of this requirement should be reported immediately to the Series Manager.

5. **Community Assets**

   1. Cosmos Projects will hold title to all trade or service marks used by the Project ("Project Trademarks"), whether based on common law or registered rights. Project Trademarks will be transferred and assigned to Cosmos Projects to hold on behalf of the Project. Any use of any Project Trademarks by Collaborators in the Project will be in accordance with the license from Cosmos Projects and inure to the benefit of Cosmos Projects.

   2. The Project will, as permitted and in accordance with such license from Cosmos Projects, develop and own all Project GitHub and social media accounts, and domain name registrations created by the Project community.

   3. Under no circumstances will Cosmos Projects be expected or required to undertake any action on behalf of the Project that is inconsistent with the tax-exempt status or purpose, as applicable, of Cosmos Projects, LLC.

6. **General Rules and Operations**

   1. The Project will:

      1. engage in the work of the Project in a professional manner consistent with maintaining a cohesive community, while also maintaining the goodwill and esteem of Cosmos Projects and other partner organizations in the open source community; and

      2. respect the rights of all trademark owners, including any branding and trademark usage guidelines.

7. **Intellectual Property Policy**

   1. Collaborators acknowledge that the copyright in all new contributions will be retained by the copyright holder as independent works of authorship and that no contributor or copyright holder will be required to assign copyrights to the Project.

   2. Except as described in Section 7.c., all contributions to the Project are subject to the following:

      1. All new inbound code contributions to the Project must be made using Apache License, Version 2.0 available at [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0) (the "Project License").

      2. All new inbound code contributions must also be accompanied by a Developer Certificate of Origin ([http://developercertificate.org](http://developercertificate.org)) sign-off in the source code system that is submitted through a contribution process approved by the TSC which will bind the authorized contributor and, if not self-employed, their employer to the applicable license.

      3. All outbound code will be made available under the Project License.

      4. Documentation will be received and made available by the Project under the Creative Commons Attribution 4.0 International License (available at [http://creativecommons.org/licenses/by/4.0/](http://creativecommons.org/licenses/by/4.0/)).

      5. The Project may seek to integrate and contribute back to other open source projects ("Upstream Projects"). In such cases, the Project will conform to all license requirements of the Upstream Projects, including dependencies, leveraged by the Project. Upstream Project code contributions not stored within the Project's main code repository will comply with the contribution process and license terms for the applicable Upstream Project.

   3. The TSC may approve the use of an alternative license or licenses for inbound or outbound contributions on an exception basis. To request an exception, please describe the contribution, the alternative open source license(s), and the justification for using an alternative open source license for the Project. License exceptions must be approved by a two-thirds vote of all of the voting members of the TSC. Contributions to the Project's specification repositories are made under the Community Specification License 1.0 as an approved exception under this Section.

   4. Contributed files should contain license information, such as SPDX short form identifiers, indicating the open source license or licenses pertaining to the file.

8. **Amendments**

   1. This Charter may be amended by a two-thirds vote of all of the voting members of the TSC and is subject to approval by Cosmos Projects.
