use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::series::Series;
use thoth_api::model::series::SeriesType;
use uuid::Uuid;

const CREATE_SERIES_MUTATION: &str = "
    mutation CreateSeries(
            $seriesType: SeriesType!,
            $seriesName: String!,
            $issnPrint: String,
            $issnDigital: String,
            $seriesUrl: String,
            $seriesDescription: String,
            $seriesCfpUrl: String,
            $imprintId: Uuid!
    ) {
        createSeries(data: {
            seriesType: $seriesType
            seriesName: $seriesName
            issnPrint: $issnPrint
            issnDigital: $issnDigital
            seriesUrl: $seriesUrl
            seriesDescription: $seriesDescription
            seriesCfpUrl: $seriesCfpUrl
            imprintId: $imprintId
        }){
            seriesId
            seriesType
            seriesName
            imprintId
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    CreateSeriesRequest,
    CreateSeriesRequestBody,
    Variables,
    CREATE_SERIES_MUTATION,
    CreateSeriesResponseBody,
    CreateSeriesResponseData,
    PushCreateSeries,
    PushActionCreateSeries
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: Option<String>,
    pub issn_digital: Option<String>,
    pub series_url: Option<String>,
    pub series_description: Option<String>,
    pub series_cfp_url: Option<String>,
    pub imprint_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeriesResponseData {
    pub create_series: Option<Series>,
}
